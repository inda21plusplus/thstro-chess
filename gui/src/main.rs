use bevy::prelude::*;
use chess_engine::{board::SquareSpec, game::Game};
use std::collections::HashMap;

type PieceAssetMap = HashMap<chess_engine::piece::Piece, Handle<ColorMaterial>>;
struct PieceSprite;
struct BoardUpdateEvent;

fn main() {
    App::build()
        // Resources
        .insert_resource(WindowDescriptor {
            title: "Chess? Yes!".into(),
            ..Default::default()
        })
        .insert_resource(Game::new())
        .init_resource::<PieceAssetMap>()
        // Event types
        .add_event::<BoardUpdateEvent>()
        // Plugins
        .add_plugins(DefaultPlugins)
        // Startup systems
        .add_startup_system(load_assets.system())
        .add_startup_system(setup_game_ui.system())
        // Systems
        .add_system(assign_square_sprites.system())
        //
        .run();
}

fn load_assets(
    asset_server: Res<AssetServer>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    mut asset_map: ResMut<PieceAssetMap>,
) {
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
            let material = materials.add(asset_server.load(path.as_str()).into());
            asset_map.insert(chess_engine::piece::Piece { color, piece }, material);
        }
    }
}

fn assign_square_sprites(
    mut commands: Commands,
    cells: Query<(Entity, &SquareSpec)>,
    sprites: Query<(Entity, &PieceSprite)>,
    chess_game: Res<Game>,
    asset_map: Res<PieceAssetMap>,
    mut board_update_event: EventReader<BoardUpdateEvent>,
) {
    for _ in board_update_event.iter() {
        for (entity, _) in sprites.iter() {
            commands.entity(entity).despawn();
        }

        for (entity, &square_spec) in cells.iter() {
            if let Some(piece) = chess_game.current_board()[square_spec] {
                commands.entity(entity).with_children(|parent| {
                    parent
                        .spawn_bundle(NodeBundle {
                            style: Style {
                                size: Size::new(Val::Percent(100.0), Val::Percent(100.0)),
                                ..Default::default()
                            },
                            material: asset_map.get(&piece).unwrap().clone(),
                            ..Default::default()
                        })
                        .insert(PieceSprite);
                });
            }
        }
    }
}

fn setup_game_ui(
    mut commands: Commands,
    mut board_update_event: EventWriter<BoardUpdateEvent>,
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
                        side_proxy.spawn_bundle(NodeBundle {
                            style: Style {
                                position_type: PositionType::Relative,
                                position: Rect {
                                    left: Val::Px(40.0),
                                    ..Default::default()
                                },
                                size: Size::new(Val::Px(300.0), Val::Percent(100.0)),
                                ..Default::default()
                            },
                            material: materials.add(Color::rgb_u8(30, 30, 30).into()),
                            ..Default::default()
                        });
                    });
                // grid
                let white_square_material = materials.add(Color::rgb_u8(50, 50, 50).into());
                let black_square_material = materials.add(Color::rgb_u8(40, 40, 40).into());
                for rank in 0..8 {
                    for file in 0..8 {
                        let material = if (rank + file) % 2 == 1 {
                            &white_square_material
                        } else {
                            &black_square_material
                        };
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
                                material: material.clone(),
                                ..Default::default()
                            })
                            .insert(SquareSpec::new(rank, file));
                    }
                }
            });
        });

    board_update_event.send(BoardUpdateEvent);
}

fn _setup(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    // ui camera
    commands.spawn_bundle(UiCameraBundle::default());
    // root node
    commands
        .spawn_bundle(NodeBundle {
            style: Style {
                size: Size::new(Val::Percent(100.0), Val::Percent(100.0)),
                justify_content: JustifyContent::SpaceBetween,
                ..Default::default()
            },
            material: materials.add(Color::NONE.into()),
            ..Default::default()
        })
        .with_children(|root| {
            // left vertical fill (border)
            root.spawn_bundle(NodeBundle {
                style: Style {
                    size: Size::new(Val::Px(200.0), Val::Percent(100.0)),
                    border: Rect::all(Val::Px(2.0)),
                    ..Default::default()
                },
                material: materials.add(Color::rgb(0.65, 0.65, 0.65).into()),
                ..Default::default()
            })
            .with_children(|left_panel| {
                // left vertical fill (content)
                left_panel
                    .spawn_bundle(NodeBundle {
                        style: Style {
                            size: Size::new(Val::Percent(100.0), Val::Percent(100.0)),
                            align_items: AlignItems::FlexEnd,
                            ..Default::default()
                        },
                        material: materials.add(Color::rgb(0.15, 0.15, 0.15).into()),
                        ..Default::default()
                    })
                    .with_children(|left_panel_inner| {
                        // text
                        left_panel_inner.spawn_bundle(TextBundle {
                            style: Style {
                                margin: Rect::all(Val::Px(5.0)),
                                ..Default::default()
                            },
                            text: Text::with_section(
                                "Text Example",
                                TextStyle {
                                    font: asset_server.load("fonts/FiraSans-Bold.otf"),
                                    font_size: 30.0,
                                    color: Color::WHITE,
                                },
                                Default::default(),
                            ),
                            ..Default::default()
                        });
                    });
            });
            // right vertical fill
            root.spawn_bundle(NodeBundle {
                style: Style {
                    size: Size::new(Val::Px(200.0), Val::Percent(100.0)),
                    ..Default::default()
                },
                material: materials.add(Color::rgb(0.15, 0.15, 0.15).into()),
                ..Default::default()
            });
            // absolute positioning
            root.spawn_bundle(NodeBundle {
                style: Style {
                    size: Size::new(Val::Px(200.0), Val::Px(200.0)),
                    position_type: PositionType::Absolute,
                    justify_content: JustifyContent::Center,
                    align_items: AlignItems::Center,
                    position: Rect {
                        left: Val::Px(210.0),
                        bottom: Val::Px(10.0),
                        ..Default::default()
                    },
                    ..Default::default()
                },
                material: materials.add(Color::rgb(0.4, 0.4, 1.0).into()),
                ..Default::default()
            })
            .with_children(|parent| {
                parent.spawn_bundle(NodeBundle {
                    style: Style {
                        size: Size::new(Val::Percent(80.0), Val::Percent(80.0)),
                        ..Default::default()
                    },
                    material: materials.add(Color::rgb(0.8, 0.8, 1.0).into()),
                    ..Default::default()
                });
            });
            // render order test: reddest in the back, whitest in the front (flex center)
            root.spawn_bundle(NodeBundle {
                style: Style {
                    size: Size::new(Val::Percent(100.0), Val::Percent(100.0)),
                    position_type: PositionType::Absolute,
                    align_items: AlignItems::Center,
                    justify_content: JustifyContent::Center,
                    ..Default::default()
                },
                material: materials.add(Color::NONE.into()),
                ..Default::default()
            })
            .with_children(|parent| {
                parent
                    .spawn_bundle(NodeBundle {
                        style: Style {
                            size: Size::new(Val::Px(100.0), Val::Px(100.0)),
                            ..Default::default()
                        },
                        material: materials.add(Color::rgb(1.0, 0.0, 0.0).into()),
                        ..Default::default()
                    })
                    .with_children(|parent| {
                        for i in 0..8 {
                            parent.spawn_bundle(NodeBundle {
                                style: Style {
                                    size: Size::new(Val::Px(100.0), Val::Px(100.0)),
                                    position_type: PositionType::Absolute,
                                    position: Rect {
                                        left: Val::Px(20.0 * i as f32),
                                        bottom: Val::Px(20.0 * i as f32),
                                        ..Default::default()
                                    },
                                    ..Default::default()
                                },
                                material: materials.add(Color::rgba(1.0, 0.2, 0.2, 0.8).into()),
                                ..Default::default()
                            });
                        }
                    });
            });
            // bevy logo (flex center)
            root.spawn_bundle(NodeBundle {
                style: Style {
                    size: Size::new(Val::Percent(100.0), Val::Percent(100.0)),
                    position_type: PositionType::Absolute,
                    justify_content: JustifyContent::Center,
                    align_items: AlignItems::FlexEnd,
                    ..Default::default()
                },
                material: materials.add(Color::NONE.into()),
                ..Default::default()
            })
            .with_children(|parent| {
                // bevy logo (image)
                parent.spawn_bundle(ImageBundle {
                    style: Style {
                        size: Size::new(Val::Px(500.0), Val::Auto),
                        ..Default::default()
                    },
                    material: materials
                        .add(asset_server.load("branding/bevy_logo_dark_big.png").into()),
                    ..Default::default()
                });
            });
        });
}
