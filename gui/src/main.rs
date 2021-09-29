use bevy::{
    diagnostic::{Diagnostics, FrameTimeDiagnosticsPlugin},
    prelude::*,
};
use chess_engine::{board::SquareSpec, game::Game};
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
        // Event types
        .add_event::<BoardUpdateEvent>()
        // Startup systems
        // .add_startup_system(load_assets.system())
        .add_startup_system(setup_game_ui.system())
        // Systems
        .add_system(assign_square_sprites.system())
        .add_system(possible_moves_hover.system())
        .add_system(show_diagnostics.system())
        .add_system(square_state_color.system())
        .add_system(pick_up_piece.system())
        //
        .run();
}

struct PieceAssetMap(HashMap<chess_engine::piece::Piece, Handle<ColorMaterial>>);
struct PieceSprite;
struct BoardUpdateEvent;
struct DiagnosticsInfoText;
struct PickedUpPiece(Option<Entity>);

#[derive(Clone, Copy)]
enum ChessSquare {
    Normal,
    Movable,
    Capturable,
}

fn pick_up_piece(
    mut query: Query<(&Interaction, &SquareSpec), Changed<Interaction>>,
    chess_game: Res<Game>,
) {
    for (&interaction, &sq_spec) in query.iter() {
        if interaction == Interaction::Clicked {
            println!("{}", sq_spec);
        }
    }
}

fn possible_moves_hover(
    piece_query: Query<(&Interaction, &SquareSpec), Changed<Interaction>>,
    mut square_query: Query<(&SquareSpec, &mut ChessSquare)>,
    chess_game: Res<Game>,
    picked_up_piece: Res<PickedUpPiece>,
) {
    if picked_up_piece.0.is_some() {
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
    let moves = chess_game.current_board().get_legal_moves(hovered);
    let destinations: HashSet<SquareSpec> = moves
        .iter()
        .filter_map(|m| match m {
            chess_engine::board::Move::Normal { to, .. } => Some(*to),
            _ => None,
        })
        .collect();

    for (&sq_spec, mut chess_square) in square_query.iter_mut() {
        if destinations.contains(&sq_spec) {
            if chess_game.current_board()[sq_spec].is_some() {
                *chess_square = ChessSquare::Capturable;
            } else {
                *chess_square = ChessSquare::Movable;
            }
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
            (true, ChessSquare::Movable) => Color::rgb_u8(0xd0, 0x87, 0x70),
            (false, ChessSquare::Movable) => Color::rgb_u8(0xbf, 0x61, 0x6a),
            (true, ChessSquare::Capturable) => Color::rgb_u8(0xeb, 0xcb, 0x8b),
            (false, ChessSquare::Capturable) => Color::rgb_u8(0xeb, 0xcb, 0x8b),
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
                                ..Default::default()
                            },
                            material: asset_map.0.get(&piece).unwrap().clone(),
                            ..Default::default()
                        })
                        .insert(Interaction::default())
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
) {
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
                    aspect_ratio: Some(0.8),
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
                            .insert(SquareSpec::new(rank, file))
                            .insert(ChessSquare::Normal);
                    }
                }
            });
        });

    board_update_event.send(BoardUpdateEvent);
}
