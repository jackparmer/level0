use bevy::prelude::*;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_systems(Startup, setup_scene)
        .add_systems(Update, rotate_camera)
        .run();
}

fn setup_scene(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    // Add a cube
    commands.spawn((
        Mesh3d(meshes.add(Cuboid::new(2.0, 2.0, 2.0))),
        MaterialMesh3d {
            material: materials.add(Color::srgb(0.8, 0.7, 0.6)),
            ..default()
        },
        Transform::from_xyz(0.0, 0.0, 0.0),
    ));

    // Add a plane
    commands.spawn((
        Mesh3d(meshes.add(Plane3d::default().mesh().size(20.0, 20.0))),
        MaterialMesh3d {
            material: materials.add(Color::srgb(0.3, 0.5, 0.3)),
            ..default()
        },
        Transform::from_xyz(0.0, -2.0, 0.0),
    ));

    // Add some additional cubes for depth
    for i in -3..=3 {
        for j in -3..=3 {
            if i == 0 && j == 0 { continue; } // Skip center
            commands.spawn((
                Mesh3d(meshes.add(Cuboid::new(1.0, 1.0, 1.0))),
                MaterialMesh3d {
                    material: materials.add(Color::srgb(
                        0.5 + (i as f32 * 0.1),
                        0.5 + (j as f32 * 0.1),
                        0.7,
                    )),
                    ..default()
                },
                Transform::from_xyz(i as f32 * 3.0, 0.0, j as f32 * 3.0),
            ));
        }
    }

    // Add lighting
    commands.spawn((
        DirectionalLight {
            shadows_enabled: true,
            ..default()
        },
        Transform::from_rotation(Quat::from_euler(
            EulerRot::ZYX,
            0.0,
            45.0_f32.to_radians(),
            -45.0_f32.to_radians(),
        )),
    ));

    // Add ambient light
    commands.insert_resource(AmbientLight {
        color: Color::WHITE,
        brightness: 0.3,
        affects_lightmapped_meshes: true,
    });

    // Add camera
    commands.spawn((
        Camera3d {
            ..default()
        },
        Transform::from_xyz(0.0, 5.0, 10.0)
            .looking_at(Vec3::new(0.0, 0.0, 0.0), Vec3::Y),
        CameraController,
    ));
}

#[derive(Component)]
struct CameraController;

fn rotate_camera(
    time: Res<Time>,
    mut camera_query: Query<&mut Transform, (With<Camera3d>, With<CameraController>)>,
) {
    for mut transform in camera_query.iter_mut() {
        let radius = 15.0;
        let height = 8.0;
        let angle = time.elapsed_secs() * 0.3;
        
        transform.translation = Vec3::new(
            angle.cos() * radius,
            height,
            angle.sin() * radius,
        );
        transform.look_at(Vec3::ZERO, Vec3::Y);
    }
}
