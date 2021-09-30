use bevy::{
    diagnostic::{Diagnostics, FrameTimeDiagnosticsPlugin},
    prelude::*,
    render::camera::Camera,
    ui::FocusPolicy,
};
use chess_engine::{
    board::{SquareDiff, SquareSpec},
    game::Game,
    piece::{Piece, PieceType},
};
use std::collections::{HashMap, HashSet};

fn main() {
    App::build()
        .insert_resource(WindowDescriptor {
            title: "Chess? Yes!".into(),
            ..Default::default()
        })
        .insert_resource(Msaa { samples: 2 })
        // Plugins
        .add_plugins(DefaultPlugins)
        .add_plugin(FrameTimeDiagnosticsPlugin::default())
        // Resources
        .insert_resource(Game::new())
        .init_resource::<PieceAssetMap>()
        .insert_resource(UIState::Default)
        // Event types
        .add_event::<BoardUpdateEvent>()
        // Startup systems
        .add_startup_system(setup_game_ui.system())
        // Systems
        .add_system(assign_square_sprites.system())
        .add_system(possible_moves_hover.system())
        .add_system(show_diagnostics.system())
        .add_system(square_state_color.system())
        .add_system(pick_up_piece.system())
        .add_system(put_down_piece.system())
        .add_system(move_picked_up_piece_to_cursor.system())
        .add_system(cancel_move.system())
        .add_system(get_pawn_promotion.system())
        //
        .run();
}

struct PieceAssetMap(HashMap<Piece, Handle<ColorMaterial>>);
struct PieceSprite;
#[derive(Clone, Copy)]
struct PawnPromotionOption(PieceType);
struct BoardUpdateEvent;
struct DiagnosticsInfoText;
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum UIState {
    Default,
    PickedUpPiece(Entity),
    PromotionAsked(SquareSpec, SquareSpec),
}
struct PickedUpPieceParent(Entity);
struct PawnPromotionElement(Entity);

#[derive(Clone, Copy)]
enum ChessSquare {
    Normal,
    Movable,
    Capturable,
    Castlable,
    Promotable,
}

fn other_color(color: Option<chess_engine::piece::Color>) -> Option<chess_engine::piece::Color> {
    match color {
        Some(chess_engine::piece::Color::White) => Some(chess_engine::piece::Color::Black),
        Some(chess_engine::piece::Color::Black) => Some(chess_engine::piece::Color::White),
        None => None,
    }
}

fn get_pawn_promotion(
    mut state: ResMut<UIState>,
    mut game: ResMut<Game>,
    pawn_promotion_element: Res<PawnPromotionElement>,
    mut pawn_promotion_element_query: Query<(&mut Style, &Children)>,
    pawn_promotion_options_query: Query<(&Interaction, &PawnPromotionOption)>,
    mut board_update_event: EventWriter<BoardUpdateEvent>,
) {
    let (from, to) = if let UIState::PromotionAsked(from, to) = *state {
        pawn_promotion_element_query
            .get_mut(pawn_promotion_element.0)
            .unwrap()
            .0
            .display = Display::Flex;
        (from, to)
    } else {
        pawn_promotion_element_query
            .get_mut(pawn_promotion_element.0)
            .unwrap()
            .0
            .display = Display::None;
        return;
    };

    let mut chosen = PieceType::Pawn;
    for (&interaction, &piece) in pawn_promotion_options_query.iter() {
        if interaction == Interaction::Clicked {
            chosen = piece.0;
        }
    }

    if chosen == PieceType::Pawn {
        return;
    }

    assert_eq!(
        game.current_board().turn(),
        game.current_board()[from].unwrap().color
    );

    game.make_move(chess_engine::board::Move::Promotion {
        from,
        to,
        target: chosen,
    });

    *state = UIState::Default;
    board_update_event.send(BoardUpdateEvent);
}

fn move_picked_up_piece_to_cursor(
    picked_up_piece_parent: Res<PickedUpPieceParent>,
    mut picked_up_piece_parent_query: Query<&mut Style>,
    windows: Res<Windows>,
    cam_query: Query<&Transform, With<Camera>>,
) {
    let window = windows.get_primary().unwrap();

    if let Some(pos) = window.cursor_position() {
        let window_height = window.height();
        let side_lenght = window_height * 0.8 / 8.0;

        let cam_tranform = cam_query.single().unwrap();
        let pos = cam_tranform.compute_matrix() * pos.extend(0.0).extend(1.0);
        let pos = Vec2::new(pos.x, pos.y);

        let mut style = picked_up_piece_parent_query
            .get_mut(picked_up_piece_parent.0)
            .unwrap();

        style.position.left = Val::Px(pos.x - side_lenght / 2.0);
        style.position.bottom = Val::Px(pos.y - side_lenght / 2.0);
        style.size = Size {
            width: Val::Px(side_lenght),
            height: Val::Px(side_lenght),
        }
    }
}

fn pick_up_piece(
    mut commands: Commands,
    query: Query<(Entity, &Interaction, &SquareSpec), (Changed<Interaction>, With<PieceSprite>)>,
    mut fp_query: Query<&mut FocusPolicy, With<PieceSprite>>,
    chess_game: Res<Game>,
    mut state: ResMut<UIState>,
    picked_up_piece_parent: Res<PickedUpPieceParent>,
) {
    if *state != UIState::Default {
        return;
    }

    for (entity, &interaction, &sq_spec) in query.iter() {
        if interaction != Interaction::Clicked {
            continue;
        }
        if Some(chess_game.current_board().turn())
            != chess_game.current_board()[sq_spec].map(|p| p.color)
        {
            continue;
        }
        for mut focus_p in fp_query.iter_mut() {
            *focus_p = FocusPolicy::Pass;
        }
        commands
            .entity(entity)
            .remove::<Parent>()
            .insert(Parent(picked_up_piece_parent.0));
        *state = UIState::PickedUpPiece(entity)
    }
}

fn put_down_piece(
    query: Query<(&Interaction, &SquareSpec), With<ChessSquare>>,
    mut state: ResMut<UIState>,
    picked_up_piece_query: Query<&SquareSpec, Without<ChessSquare>>,
    mut chess_game: ResMut<Game>,
    mut board_update_event: EventWriter<BoardUpdateEvent>,
) {
    let piece = match *state {
        UIState::PickedUpPiece(p) => p,
        _ => return,
    };
    let from_sq = *picked_up_piece_query.get(piece).unwrap();
    let mut target = None;
    for (&interaction, &sq_spec) in query.iter() {
        if interaction == Interaction::Clicked {
            target = Some(sq_spec);
        }
    }
    let dst_sq = match target {
        Some(t) => t,
        None => return,
    };

    let color = chess_game.current_board()[from_sq].map(|p| p.color);

    // Promotion
    if Some(dst_sq.rank) == other_color(color).map(|c| c.home_rank()) {
        *state = UIState::PromotionAsked(from_sq, dst_sq);
        board_update_event.send(BoardUpdateEvent);
    } else if dst_sq == from_sq
        || chess_game
            .make_move(chess_engine::board::Move::Normal {
                from: from_sq,
                to: dst_sq,
            })
            .is_some()
        // Castling
        || dst_sq - from_sq == SquareDiff::new(0, -2)
            && chess_game.current_board()[from_sq].map(|p| p.piece)
                == Some(chess_engine::piece::PieceType::King)
            && chess_game
                .make_move(chess_engine::board::Move::Castling(
                    chess_engine::board::Castling::Long,
                ))
                .is_some()
        || dst_sq - from_sq == SquareDiff::new(0, 2)
            && chess_game.current_board()[from_sq].map(|p| p.piece)
                == Some(chess_engine::piece::PieceType::King)
            && chess_game
                .make_move(chess_engine::board::Move::Castling(
                    chess_engine::board::Castling::Short,
                ))
                .is_some()
    {
        *state = UIState::Default;
        board_update_event.send(BoardUpdateEvent);
    }
}

fn cancel_move(
    mouse_input: Res<Input<MouseButton>>,
    kb_input: Res<Input<KeyCode>>,
    mut state: ResMut<UIState>,
    // picked_up_piece_query: Query<&SquareSpec, Without<ChessSquare>>,
    // square_query: Query<&SquareSpec, With<ChessSquare>>,
    mut board_update_event: EventWriter<BoardUpdateEvent>,
) {
    if !(mouse_input.just_pressed(MouseButton::Right) || kb_input.just_pressed(KeyCode::Escape)) {
        return;
    }
    *state = UIState::Default;
    board_update_event.send(BoardUpdateEvent);
}

fn possible_moves_hover(
    piece_query: Query<(&Interaction, &SquareSpec), Changed<Interaction>>,
    mut square_query: Query<(&SquareSpec, &mut ChessSquare)>,
    chess_game: Res<Game>,
    state: Res<UIState>,
) {
    if *state != UIState::Default {
        return;
    }

    let mut hovered = None;
    let mut changed = false;

    for (&interaction, &sq_spec) in piece_query.iter() {
        changed = true;
        if interaction == Interaction::Hovered || interaction == Interaction::Clicked {
            hovered = Some(sq_spec);
            break;
        }
    }

    if !changed {
        return;
    }

    for (_, mut chess_square) in square_query.iter_mut() {
        *chess_square = ChessSquare::Normal;
    }
    let hovered = match hovered {
        Some(hovered) => hovered,
        None => return,
    };
    let color = chess_game.current_board()[hovered].map(|p| p.color);
    let piece = chess_game.current_board()[hovered].map(|p| p.piece);
    let moves = chess_game.current_board().get_legal_moves(hovered);
    let moves: HashSet<chess_engine::board::Move> = moves.into_iter().collect();
    let destinations: HashSet<SquareSpec> = moves
        .iter()
        .filter_map(|m| match m {
            chess_engine::board::Move::Normal { to, .. } => Some(*to),
            chess_engine::board::Move::Promotion { to, .. } => Some(*to),
            _ => None,
        })
        .collect();

    for (&sq_spec, mut chess_square) in square_query.iter_mut() {
        if destinations.contains(&sq_spec) {
            if Some(sq_spec.rank) == other_color(color).map(|c| c.home_rank())
                && piece == Some(chess_engine::piece::PieceType::Pawn)
            {
                *chess_square = ChessSquare::Promotable;
            } else if chess_game.current_board()[sq_spec].is_some() {
                *chess_square = ChessSquare::Capturable;
            } else {
                *chess_square = ChessSquare::Movable;
            }
        } else if (sq_spec - hovered) == SquareDiff::new(0, -2)
            && moves.contains(&chess_engine::board::Move::Castling(
                chess_engine::board::Castling::Long,
            ))
            || (sq_spec - hovered) == SquareDiff::new(0, 2)
                && moves.contains(&chess_engine::board::Move::Castling(
                    chess_engine::board::Castling::Short,
                ))
        {
            *chess_square = ChessSquare::Castlable;
        }
    }
}

// TODO: cache materials
fn square_state_color(
    mut query: Query<(&SquareSpec, &ChessSquare, &mut Handle<ColorMaterial>), Changed<ChessSquare>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    for (&sq_spec, &chess_square, mut material) in query.iter_mut() {
        let is_white = (sq_spec.file + sq_spec.rank) % 2 == 1;
        let color = match (is_white, chess_square) {
            (true, ChessSquare::Normal) => Color::rgb_u8(50, 50, 50),
            (false, ChessSquare::Normal) => Color::rgb_u8(40, 40, 40),
            (true, ChessSquare::Capturable) => Color::rgb_u8(0xd0, 0x87, 0x70),
            (false, ChessSquare::Capturable) => Color::rgb_u8(0xbf, 0x61, 0x6a),
            (true, ChessSquare::Movable) => Color::rgb_u8(0xdb, 0xbb, 0x7b),
            (false, ChessSquare::Movable) => Color::rgb_u8(0xca, 0xa1, 0x75),
            (_, ChessSquare::Castlable) => Color::rgb_u8(0x8f, 0xbc, 0xbb),
            (true, ChessSquare::Promotable) => Color::rgb_u8(0x81, 0xa1, 0xc1),
            (false, ChessSquare::Promotable) => Color::rgb_u8(0x5e, 0x81, 0xac),
        };
        *material = materials.add(color.into());
    }
}

fn show_diagnostics(
    diagnostics: Res<Diagnostics>,
    mut query: Query<&mut Text, With<DiagnosticsInfoText>>,
) {
    if let Some(fps) = diagnostics
        .get(FrameTimeDiagnosticsPlugin::FPS)
        .unwrap()
        .average()
    {
        let mut text = query.single_mut().unwrap();
        text.sections[0].value = format!("Fps: {:.0}", fps);
    }
}

impl FromWorld for PieceAssetMap {
    fn from_world(world: &mut World) -> Self {
        let mut this = HashMap::default();
        let asset_server = world.get_resource::<AssetServer>().unwrap();
        let mut assets = vec![];
        for (color, color_ch) in [
            (chess_engine::piece::Color::White, 'w'),
            (chess_engine::piece::Color::Black, 'b'),
        ] {
            for (piece, pt_ch) in [
                (chess_engine::piece::PieceType::Bishop, 'b'),
                (chess_engine::piece::PieceType::King, 'k'),
                (chess_engine::piece::PieceType::Knight, 'n'),
                (chess_engine::piece::PieceType::Pawn, 'p'),
                (chess_engine::piece::PieceType::Queen, 'q'),
                (chess_engine::piece::PieceType::Rook, 'r'),
            ] {
                let path = format!("pieces/{}{}.png", color_ch, pt_ch);
                assets.push((
                    chess_engine::piece::Piece { color, piece },
                    asset_server.load(path.as_str()),
                ));
            }
        }
        let mut materials = world.get_resource_mut::<Assets<ColorMaterial>>().unwrap();
        for (piece, asset) in assets {
            let material = materials.add(asset.into());
            this.insert(piece, material);
        }
        Self(this)
    }
}

fn assign_square_sprites(
    mut commands: Commands,
    cells: Query<(Entity, &SquareSpec), With<ChessSquare>>,
    sprites: Query<(Entity, &PieceSprite)>,
    chess_game: Res<Game>,
    asset_map: Res<PieceAssetMap>,
    mut board_update_event: EventReader<BoardUpdateEvent>,
) {
    for _ in board_update_event.iter() {
        for (entity, _) in sprites.iter() {
            commands.entity(entity).despawn();
        }

        for (entity, &sq_spec) in cells.iter() {
            if let Some(piece) = chess_game.current_board()[sq_spec] {
                commands.entity(entity).with_children(|parent| {
                    parent
                        .spawn_bundle(NodeBundle {
                            style: Style {
                                size: Size::new(Val::Percent(100.0), Val::Percent(100.0)),
                                position_type: PositionType::Absolute,
                                ..Default::default()
                            },
                            material: asset_map.0.get(&piece).unwrap().clone(),
                            ..Default::default()
                        })
                        .insert(Interaction::default())
                        .insert(FocusPolicy::Block)
                        .insert(sq_spec.clone())
                        .insert(PieceSprite);
                });
            }
        }
    }
}

fn setup_game_ui(
    mut commands: Commands,
    mut board_update_event: EventWriter<BoardUpdateEvent>,
    asset_server: Res<AssetServer>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    piece_asset_map: Res<PieceAssetMap>,
) {
    let mut picked_up_piece_parent = Entity::new(0);
    let mut pawn_promotion_element = Entity::new(0);
    commands.spawn_bundle(UiCameraBundle::default());
    commands
        .spawn_bundle(NodeBundle {
            style: Style {
                size: Size::new(Val::Percent(100.0), Val::Percent(100.0)),
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                ..Default::default()
            },
            material: materials.add(Color::rgb_u8(20, 20, 20).into()),
            ..Default::default()
        })
        .with_children(|root| {
            root.spawn_bundle(NodeBundle {
                style: Style {
                    size: Size::new(Val::Undefined, Val::Percent(80.0)),
                    aspect_ratio: Some(0.8), // i dont know why but this makes it a square
                    ..Default::default()
                },
                material: materials.add(Color::rgb_u8(40, 40, 40).into()),
                ..Default::default()
            })
            .with_children(|board| {
                board
                    .spawn_bundle(NodeBundle {
                        style: Style {
                            position_type: PositionType::Relative,
                            position: Rect {
                                left: Val::Percent(100.0),
                                ..Default::default()
                            },
                            ..Default::default()
                        },
                        material: materials.add(Color::NONE.into()),
                        ..Default::default()
                    })
                    .with_children(|side_proxy| {
                        side_proxy
                            .spawn_bundle(NodeBundle {
                                style: Style {
                                    position_type: PositionType::Relative,
                                    position: Rect {
                                        left: Val::Px(40.0),
                                        ..Default::default()
                                    },
                                    size: Size::new(Val::Px(200.0), Val::Percent(100.0)),
                                    ..Default::default()
                                },
                                material: materials.add(Color::rgb_u8(30, 30, 30).into()),
                                ..Default::default()
                            })
                            .with_children(|side_panel| {
                                side_panel
                                    .spawn_bundle(TextBundle {
                                        text: Text::with_section(
                                            "",
                                            TextStyle {
                                                font: asset_server.load("fonts/FiraSans-Bold.otf"),
                                                font_size: 12.0,
                                                color: Color::WHITE,
                                            },
                                            Default::default(),
                                        ),
                                        ..Default::default()
                                    })
                                    .insert(DiagnosticsInfoText);
                            });
                    });
                // grid
                for rank in 0..8 {
                    for file in 0..8 {
                        board
                            .spawn_bundle(NodeBundle {
                                style: Style {
                                    position_type: PositionType::Absolute,
                                    position: Rect {
                                        bottom: Val::Percent(rank as f32 * 100.0 / 8.0),
                                        left: Val::Percent(file as f32 * 100.0 / 8.0),
                                        ..Default::default()
                                    },
                                    size: Size::new(
                                        Val::Percent(100.0 / 8.0),
                                        Val::Percent(100.0 / 8.0),
                                    ),
                                    ..Default::default()
                                },
                                ..Default::default()
                            })
                            .insert(Interaction::default())
                            .insert(SquareSpec::new(rank, file))
                            .insert(ChessSquare::Normal);
                    }
                }
            });
            picked_up_piece_parent = root
                .spawn_bundle(NodeBundle {
                    style: Style {
                        position_type: PositionType::Absolute,
                        ..Default::default()
                    },
                    material: materials.add(Color::NONE.into()),
                    ..Default::default()
                })
                .insert(FocusPolicy::Pass)
                .id();
            pawn_promotion_element = root
                .spawn_bundle(NodeBundle {
                    style: Style {
                        position_type: PositionType::Absolute,
                        display: Display::None,
                        ..Default::default()
                    },
                    material: materials.add(Color::rgb_u8(0x88, 0xc0, 0xd0).into()),
                    ..Default::default()
                })
                .with_children(|parent| {
                    for piece in [
                        PieceType::Bishop,
                        PieceType::Knight,
                        PieceType::Queen,
                        PieceType::Rook,
                    ] {
                        parent
                            .spawn_bundle(NodeBundle {
                                style: Style {
                                    // TODO: dynamic size
                                    size: Size {
                                        width: Val::Px(80.0),
                                        height: Val::Px(80.0),
                                    },
                                    ..Default::default()
                                },
                                material: piece_asset_map.0[&Piece {
                                    piece,
                                    color: chess_engine::piece::Color::White,
                                }]
                                    .clone(),
                                ..Default::default()
                            })
                            .insert(Interaction::default())
                            .insert(PawnPromotionOption(piece));
                    }
                })
                .id();
        });

    commands.insert_resource(PickedUpPieceParent(picked_up_piece_parent));
    commands.insert_resource(PawnPromotionElement(pawn_promotion_element));

    board_update_event.send(BoardUpdateEvent);
}
