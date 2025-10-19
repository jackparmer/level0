use bevy::prelude::*;
use wasm_bindgen::prelude::*;

#[cfg(target_arch = "wasm32")]
use web_sys;
#[cfg(target_arch = "wasm32")]
use js_sys;

// Simple pseudo-random function for WASM compatibility
fn pseudo_random(seed: f32) -> f32 {
    let x = seed.sin() * 43758.5453;
    x - x.floor()
}

#[wasm_bindgen::prelude::wasm_bindgen(start)]
pub fn main() {
    App::new()
        // Removed shadow map for better performance
        .add_plugins(DefaultPlugins)
        .insert_resource(ClearColor(Color::srgb(0.005, 0.005, 0.005))) // Much darker background
        .add_systems(Startup, setup)
        .add_systems(Update, (move_cube, follow_camera, spawn_footsteps, update_footsteps, draw_wireframe, rotate_radar, spawn_spheres, chase_cube, draw_line_of_sight, despawn_spheres, update_smoke, update_health))
        .run();
}

/// set up a simple 3D scene with a single centered cube and circuit-textured base
fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    asset_server: Res<AssetServer>,
) {
    // Large square base platform with texture
    commands.spawn((
        Mesh3d(meshes.add(Rectangle::new(400.0, 400.0))), // 2x smaller
        MeshMaterial3d(materials.add(StandardMaterial {
            base_color_texture: Some(asset_server.load("textures/snow_02_diff_4k.png")),
            ..default()
        })),
        Transform::from_rotation(Quat::from_rotation_x(-std::f32::consts::FRAC_PI_2)),
        GlobalTransform::default(),
        Visibility::default(),
        InheritedVisibility::default(),
        ObstacleBlocker {
            half_size: Vec3::new(200.0, 0.1, 200.0), // Large flat collision box for the ground
        },
    ));
    
    // Greyscale cube floating above the base
    commands.spawn((
        Mesh3d(meshes.add(Cuboid::new(1.0, 1.0, 1.0))),
        MeshMaterial3d(materials.add(StandardMaterial {
            base_color: Color::srgb(0.5, 0.5, 0.5), // Greyscale
            emissive: Color::srgb(0.0, 0.0, 0.0).into(), // No emissive
            alpha_mode: AlphaMode::Blend, // Enable transparency
            ..default()
        })),
        Transform::from_xyz(0.0, 0.5, 0.0), // Half unit above the base
        GlobalTransform::default(),
        Visibility::default(),
        InheritedVisibility::default(),
        CubeController, // Add controller component
        Health { current: 100.0, max: 100.0 }, // Add health component
    ));

    
    // Add directional light for better visibility
    commands.spawn((
        DirectionalLight {
            color: Color::srgb(1.0, 1.0, 1.0), // Bright white
            illuminance: 20000.0, // Much more intense
            shadows_enabled: true, // Enable shadows
            ..default()
        },
        Transform::from_rotation(Quat::from_euler(
            EulerRot::ZYX,
            0.0,
            0.0_f32.to_radians(), // More overhead
            -60.0_f32.to_radians(), // Steeper angle
        )),
        GlobalTransform::default(),
        Visibility::default(),
        InheritedVisibility::default(),
    ));
    
    // Create skybox with stars texture
    commands.spawn((
        Mesh3d(meshes.add(Sphere::new(1000.0))), // Large sphere for skybox
        MeshMaterial3d(materials.add(StandardMaterial {
            base_color_texture: Some(asset_server.load("textures/stars.png")),
            unlit: true, // Skybox should not be affected by lighting
            ..default()
        })),
        Transform::from_scale(Vec3::splat(-1.0)), // Invert the sphere so we see the inside
        GlobalTransform::default(),
        Visibility::default(),
        InheritedVisibility::default(),
    ));

    // Add industrial building in the distance
    let building_angle = pseudo_random(300.0) * 2.0 * std::f32::consts::PI;
    let building_radius = pseudo_random(301.0) * 80.0 + 100.0; // Far distance: 100-180 units
    let building_x = building_angle.cos() * building_radius;
    let building_z = building_angle.sin() * building_radius;
    let building_scale = pseudo_random(302.0) * 0.1 + 0.05; // Small scale for distant building
    let building_rotation = pseudo_random(303.0) * 2.0 * std::f32::consts::PI;
    
    // Spawn the building GLB model
    commands.spawn((
        SceneRoot(asset_server.load("models/building/industrialbuildingpart.gltf#Scene0")),
        Transform::from_scale(Vec3::splat(building_scale))
            .with_translation(Vec3::new(building_x, 0.0, building_z))
            .with_rotation(Quat::from_rotation_y(building_rotation)),
        GlobalTransform::default(),
        Visibility::default(),
        InheritedVisibility::default(),
        ObstacleBlocker {
            half_size: Vec3::new(10.0, 7.5, 10.0), // Building collision box half-size
        },
    ));
    
    // Add invisible collision mesh for the building
    commands.spawn((
        Mesh3d(meshes.add(Cuboid::new(20.0, 15.0, 20.0))), // Invisible collision box
        MeshMaterial3d(materials.add(StandardMaterial {
            base_color: Color::srgba(0.0, 0.0, 0.0, 0.0), // Completely transparent
            alpha_mode: AlphaMode::Mask(0.5), // Use mask mode for reliable transparency
            unlit: true, // Don't affect lighting
            ..default()
        })),
        Transform::from_scale(Vec3::splat(building_scale))
            .with_translation(Vec3::new(building_x, 0.0, building_z)) // Position at ground level
            .with_rotation(Quat::from_rotation_y(building_rotation)),
        GlobalTransform::default(),
        Visibility::default(),
        InheritedVisibility::default(),
        ObstacleBlocker {
            half_size: Vec3::new(10.0, 5.0, 10.0), // Building collision box half-size (10 units tall)
        },
    ));
    
    // Add red overhead light illuminating the building
    commands.spawn((
        PointLight {
            color: Color::srgb(1.0, 0.0, 0.0), // Bright red
            intensity: 100000.0, // Much brighter
            range: 100.0, // Much wider range to illuminate building
            shadows_enabled: true,
            ..default()
        },
        Transform::from_xyz(building_x, 390.0, building_z), // Higher above the building
        GlobalTransform::default(),
        Visibility::default(),
        InheritedVisibility::default(),
    ));
    
    // Track all object positions for distance checking
    let mut all_positions = Vec::new();
    
    // Spawn all 11 glaciers (including the first one) in a consolidated loop
    for i in 0..11 {
        let mut attempts = 0;
        let mut valid_position = false;
        let mut glacier_x = 0.0;
        let mut glacier_z = 0.0;
        
        // For the first glacier (i=0), place it directly without distance checking
        if i == 0 {
            let glacier_angle = pseudo_random(400.0) * 2.0 * std::f32::consts::PI;
            let glacier_radius = pseudo_random(401.0) * 80.0 + 100.0; // Far distance: 100-180 units
            glacier_x = glacier_angle.cos() * glacier_radius;
            glacier_z = glacier_angle.sin() * glacier_radius;
            valid_position = true;
            all_positions.push((glacier_x, glacier_z));
        } else {
            // For subsequent glaciers, try to find a position that's not too close to existing objects
            while !valid_position && attempts < 50 {
                let glacier_angle = pseudo_random(500.0 + i as f32 + attempts as f32) * 2.0 * std::f32::consts::PI;
                let glacier_radius = pseudo_random(501.0 + i as f32 + attempts as f32) * 80.0 + 100.0; // Far distance: 100-180 units
                glacier_x = glacier_angle.cos() * glacier_radius;
                glacier_z = glacier_angle.sin() * glacier_radius;
                
                // Check distance from all existing object positions
                let mut too_close = false;
                for (existing_x, existing_z) in &all_positions {
                    let distance = ((glacier_x - existing_x).powi(2) + (glacier_z - existing_z).powi(2)).sqrt();
                    if distance < 25.0 { // Minimum distance of 25 units between objects
                        too_close = true;
                        break;
                    }
                }
                
                if !too_close {
                    valid_position = true;
                    all_positions.push((glacier_x, glacier_z));
                }
                
                attempts += 1;
            }
        }
        
        let glacier_scale = if i == 0 {
            pseudo_random(402.0) * 0.08 + 0.05 // Original scale for first glacier
        } else {
            pseudo_random(502.0 + i as f32) * 0.08 + 0.05 // Same small scale: 0.05-0.13
        };
        let glacier_rotation = if i == 0 {
            pseudo_random(403.0) * 2.0 * std::f32::consts::PI // Original rotation for first glacier
        } else {
            pseudo_random(503.0 + i as f32) * 2.0 * std::f32::consts::PI
        };
        let glacier_height = -4.0; // Same low height
        
        // Spawn the glacier GLB model
        commands.spawn((
            SceneRoot(asset_server.load("models/glacier/Iceberg.gltf#Scene0")),
            Transform::from_scale(Vec3::splat(glacier_scale))
                .with_translation(Vec3::new(glacier_x, glacier_height, glacier_z))
                .with_rotation(Quat::from_rotation_y(glacier_rotation)),
            GlobalTransform::default(),
            Visibility::default(),
            InheritedVisibility::default(),
            ObstacleBlocker {
            half_size: Vec3::new(10.0, 7.5, 10.0), // Building collision box half-size
        },
        )).id();
        
         // Add visible collision mesh for the glacier
         commands.spawn((
             Mesh3d(meshes.add(Cylinder::new(160.0, 80.0))), // Disc-shaped collision mesh
             MeshMaterial3d(materials.add(StandardMaterial {
                 base_color: Color::srgba(0.0, 0.0, 0.0, 0.0), // Completely transparent
                 alpha_mode: AlphaMode::Mask(0.5), // Use mask mode for reliable transparency
                 unlit: true, // Don't affect lighting
                 ..default()
             })),
             Transform::from_scale(Vec3::splat(glacier_scale))
                 .with_translation(Vec3::new(glacier_x, 0.0, glacier_z)) // Position at ground level
                 .with_rotation(Quat::from_rotation_y(glacier_rotation)),
             GlobalTransform::default(),
             Visibility::default(),
             InheritedVisibility::default(),
             ObstacleBlocker {
                 half_size: Vec3::new(80.0, 40.0, 80.0), // 10x larger glacier collision box half-size (80 units tall)
             },
         ));
    }
    
    // Add radar model at random location with distance checking
    let mut attempts = 0;
    let mut valid_position = false;
    let mut radar_x = 0.0;
    let mut radar_z = 0.0;
    
    // Try to find a position that's not too close to existing objects
    while !valid_position && attempts < 50 {
        let radar_angle = pseudo_random(999.0 + attempts as f32) * 2.0 * std::f32::consts::PI;
        let radar_radius = pseudo_random(998.0 + attempts as f32) * 20.0 + 150.0; // Closer to outskirts: 150-170
        radar_x = radar_angle.cos() * radar_radius;
        radar_z = radar_angle.sin() * radar_radius;
        
        // Check distance from all existing object positions
        let mut too_close = false;
        for (existing_x, existing_z) in &all_positions {
            let distance = ((radar_x - existing_x).powi(2) + (radar_z - existing_z).powi(2)).sqrt();
            if distance < 25.0 { // Minimum distance of 25 units between objects
                too_close = true;
                break;
            }
        }
        
        if !too_close {
            valid_position = true;
            all_positions.push((radar_x, radar_z));
        }
        
        attempts += 1;
    }
    let radar_scale = pseudo_random(997.0) * 0.015 + 0.01; // Random scale between 0.01-0.025
    let radar_rotation = pseudo_random(996.0) * 2.0 * std::f32::consts::PI;
    
    // Spawn the first radar GLB model
    commands.spawn((
        SceneRoot(asset_server.load("models/radar/Radar_HENSOLDT_ASR_NG.gltf#Scene0")),
        Transform::from_scale(Vec3::splat(radar_scale))
            .with_translation(Vec3::new(radar_x, 0.0, radar_z))
            .with_rotation(Quat::from_rotation_y(radar_rotation)),
        GlobalTransform::default(),
        Visibility::default(),
        InheritedVisibility::default(),
        RotatingRadar, // Add rotating component
        ObstacleBlocker {
            half_size: Vec3::new(10.0, 7.5, 10.0), // Building collision box half-size
        },
    )).id();
    
    // Add invisible collision mesh for the radar
    commands.spawn((
        Mesh3d(meshes.add(Cuboid::new(3.0, 4.0, 3.0))), // Invisible collision box
        MeshMaterial3d(materials.add(StandardMaterial {
            base_color: Color::srgba(0.0, 0.0, 0.0, 0.0), // Completely transparent
            alpha_mode: AlphaMode::Mask(0.5), // Use mask mode for reliable transparency
            unlit: true, // Don't affect lighting
            ..default()
        })),
        Transform::from_scale(Vec3::splat(radar_scale))
            .with_translation(Vec3::new(radar_x, 0.0, radar_z)) // Position at ground level
            .with_rotation(Quat::from_rotation_y(radar_rotation)),
        GlobalTransform::default(),
        Visibility::default(),
        InheritedVisibility::default(),
        ObstacleBlocker {
            half_size: Vec3::new(1.5, 1.0, 1.5), // Radar collision box half-size (2 units tall)
        },
    ));

    // Add 5 more HENSOLDT radars at random positions (avoiding glacier positions and each other)
    let mut radar_positions = Vec::new();
    radar_positions.push((radar_x, radar_z)); // Add the first radar position
    
    for i in 0..5 {
        let mut attempts = 0;
        let mut valid_position = false;
        let mut radar_x = 0.0;
        let mut radar_z = 0.0;
        
        // Try to find a position that's not too close to existing radars
        while !valid_position && attempts < 50 {
            let radar_angle = pseudo_random(990.0 + i as f32 + attempts as f32) * 2.0 * std::f32::consts::PI;
            let radar_radius = pseudo_random(989.0 + i as f32 + attempts as f32) * 20.0 + 150.0; // Closer to outskirts: 150-170
            radar_x = radar_angle.cos() * radar_radius;
            radar_z = radar_angle.sin() * radar_radius;
            
            // Check distance from all existing object positions
            let mut too_close = false;
            for (existing_x, existing_z) in &all_positions {
                let distance = ((radar_x - existing_x).powi(2) + (radar_z - existing_z).powi(2)).sqrt();
                if distance < 25.0 { // Minimum distance of 25 units between objects
                    too_close = true;
                    break;
                }
            }
            
            if !too_close {
                valid_position = true;
                radar_positions.push((radar_x, radar_z));
                all_positions.push((radar_x, radar_z));
            }
            
            attempts += 1;
        }
        
        let radar_scale = pseudo_random(988.0 + i as f32) * 0.015 + 0.01; // Random scale between 0.01-0.025 (2x smaller)
        let radar_rotation = pseudo_random(987.0 + i as f32) * 2.0 * std::f32::consts::PI;
        
        // Spawn the radar GLB model
        commands.spawn((
            SceneRoot(asset_server.load("models/radar/Radar_HENSOLDT_ASR_NG.gltf#Scene0")),
            Transform::from_scale(Vec3::splat(radar_scale))
                .with_translation(Vec3::new(radar_x, 0.0, radar_z))
                .with_rotation(Quat::from_rotation_y(radar_rotation)),
            GlobalTransform::default(),
            Visibility::default(),
            InheritedVisibility::default(),
            RotatingRadar, // Add rotating component
            ObstacleBlocker {
            half_size: Vec3::new(10.0, 7.5, 10.0), // Building collision box half-size
        },
        )).id();
        
        // Add invisible collision mesh for the radar
        commands.spawn((
            Mesh3d(meshes.add(Cuboid::new(3.0, 4.0, 3.0))), // Invisible collision box
            MeshMaterial3d(materials.add(StandardMaterial {
                base_color: Color::srgba(0.0, 0.0, 0.0, 0.0), // Completely transparent
                alpha_mode: AlphaMode::Mask(0.5), // Use mask mode for reliable transparency
                unlit: true, // Don't affect lighting
                ..default()
            })),
            Transform::from_scale(Vec3::splat(radar_scale))
                .with_translation(Vec3::new(radar_x, 0.0, radar_z)) // Position at ground level
                .with_rotation(Quat::from_rotation_y(radar_rotation)),
            GlobalTransform::default(),
            Visibility::default(),
            InheritedVisibility::default(),
            ObstacleBlocker {
                half_size: Vec3::new(1.5, 1.0, 1.5), // Radar collision box half-size (2 units tall)
            },
        ));
    }
    
    // Add large rusty metal sphere at random location intersecting the plane
    let sphere_angle = pseudo_random(995.0) * 2.0 * std::f32::consts::PI;
    let sphere_radius = pseudo_random(994.0) * 30.0 + 20.0; // Random radius between 20-50
    let sphere_x = sphere_angle.cos() * sphere_radius;
    let sphere_z = sphere_angle.sin() * sphere_radius;
    let sphere_scale = pseudo_random(993.0) * 4.0 + 6.0; // Random scale between 6-10 (half size)
    
    commands.spawn((
        Mesh3d(meshes.add(Sphere::new(sphere_scale))),
        MeshMaterial3d(materials.add(StandardMaterial {
            base_color_texture: Some(asset_server.load("textures/rusty_metal_grid_diff_4k.png")),
            ..default()
        })),
        Transform::from_xyz(sphere_x, sphere_scale * 0.5, sphere_z), // Position so it intersects the plane
        GlobalTransform::default(),
        Visibility::default(),
        InheritedVisibility::default(),
        ObstacleBlocker {
            half_size: Vec3::new(sphere_scale, sphere_scale, sphere_scale), // Sphere collision box (radius = scale)
        },
    ));
    
    // Simple ambient lighting for performance
    commands.spawn(AmbientLight {
        color: Color::srgb(0.3, 0.3, 0.4), // Brighter ambient
        brightness: 0.5, // Higher brightness
        affects_lightmapped_meshes: true,
    });
    
    // Main camera positioned like the official example
    commands.spawn((
        Camera3d::default(),
        DistanceFog {
            color: Color::srgb(0.05, 0.05, 0.05), // Much lighter fog
            falloff: FogFalloff::Linear {
                start: 30.0, // Start much further away
                end: 100.0, // End much further away
            },
            ..default()
        },
        Transform::from_xyz(15.0, 5.0, 20.0).looking_at(Vec3::ZERO, Vec3::Y),
        GlobalTransform::default(),
        Visibility::default(),
        InheritedVisibility::default(),
        CameraController, // Add controller component
        CameraAngle { pitch: 0.0 }, // Add camera angle component
    ));
    
    // Add sphere spawner
    commands.spawn((
        SphereSpawner {
            timer: Timer::from_seconds(10.0, TimerMode::Repeating),
        },
    ));
    
}

#[derive(Component)]
struct CubeController;

#[derive(Component)]
struct Health {
    current: f32,
    max: f32,
}



#[derive(Component)]
struct CameraController;

#[derive(Component)]
struct RotatingRadar;

#[derive(Component)]
struct BlinkingLight;

#[derive(Component)]
struct CameraAngle {
    pitch: f32,
}

#[derive(Component)]
struct StrobingLight;

#[derive(Component)]
struct Footstep {
    lifetime: f32,
    max_lifetime: f32,
}

#[derive(Component)]
struct ObstacleBlocker {
    half_size: Vec3,
}

#[derive(Component)]
struct ChasingSphere {
    speed: f32,
    last_line_of_sight: bool,
}

#[derive(Component)]
struct SphereSpawner {
    timer: Timer,
}

#[derive(Component)]
struct SmokeParticle {
    lifetime: f32,
    max_lifetime: f32,
    velocity: Vec3,
}

fn move_cube(
    keyboard_input: Res<ButtonInput<KeyCode>>,
    time: Res<Time>,
    mut cube_query: Query<&mut Transform, With<CubeController>>,
) {
    for mut transform in cube_query.iter_mut() {
        let base_speed = 5.0;
        let speed = if keyboard_input.pressed(KeyCode::ShiftLeft) || keyboard_input.pressed(KeyCode::ShiftRight) {
            base_speed * 2.0 // 2x faster when shift is held
        } else {
            base_speed
        };
        let rotation_speed = 2.0;
        let dt = time.delta_secs();
        
        // W and S for forward/backward movement in the cube's local direction
        if keyboard_input.pressed(KeyCode::KeyW) {
            let forward = transform.forward();
            transform.translation += forward * speed * dt;
        }
        if keyboard_input.pressed(KeyCode::KeyS) {
            let backward = transform.back();
            transform.translation += backward * speed * dt;
        }
        
        // A and D for rotation (instead of horizontal movement)
        if keyboard_input.pressed(KeyCode::KeyA) {
            transform.rotate_y(rotation_speed * dt);
        }
        if keyboard_input.pressed(KeyCode::KeyD) {
            transform.rotate_y(-rotation_speed * dt);
        }
        
        // Q and E for additional rotation
        if keyboard_input.pressed(KeyCode::KeyQ) {
            transform.rotate_y(rotation_speed * dt);
        }
        if keyboard_input.pressed(KeyCode::KeyE) {
            transform.rotate_y(-rotation_speed * dt);
        }
    }
}

fn follow_camera(
    keyboard_input: Res<ButtonInput<KeyCode>>,
    time: Res<Time>,
    cube_query: Query<&Transform, (With<CubeController>, Without<CameraController>)>,
    mut camera_query: Query<(&mut Transform, &mut CameraAngle), (With<CameraController>, Without<CubeController>)>,
) {
    if let Ok(cube_transform) = cube_query.single() {
        for (mut camera_transform, mut camera_angle) in camera_query.iter_mut() {
            // Handle camera angle adjustment with up/down arrow keys
            let angle_speed = 2.0;
            let dt = time.delta_secs();
            
            if keyboard_input.pressed(KeyCode::ArrowUp) {
                camera_angle.pitch += angle_speed * dt;
            }
            if keyboard_input.pressed(KeyCode::ArrowDown) {
                camera_angle.pitch -= angle_speed * dt;
            }
            
            // Clamp pitch angle to reasonable limits
            camera_angle.pitch = camera_angle.pitch.clamp(-1.5, 1.5);
            
            // Follow behind the cube at a greater distance and height for a steeper angle
            let follow_distance = 16.0;
            let follow_height = 10.0;
            
            // Calculate the position behind the cube based on its rotation
            let behind_offset = cube_transform.back() * follow_distance;
            let height_offset = Vec3::new(0.0, follow_height, 0.0);
            
            // Set camera position behind and above the cube
            camera_transform.translation = cube_transform.translation + behind_offset + height_offset;
            
            // Apply pitch angle to camera look direction
            let look_target = cube_transform.translation + Vec3::new(0.0, camera_angle.pitch * 5.0, 0.0);
            camera_transform.look_at(look_target, Vec3::Y);
        }
    }
}

// Removed strobing system for better performance

/// System to spawn footsteps behind the cube as it moves
fn spawn_footsteps(
    time: Res<Time>,
    cube_query: Query<&Transform, With<CubeController>>,
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    if let Ok(cube_transform) = cube_query.single() {
        // Spawn a footstep every 0.3 seconds when moving
        let _spawn_interval = 0.3;
        let current_time = time.elapsed_secs();
        
        // Simple spawn logic - spawn footsteps periodically
        if (current_time * 10.0) as i32 % 3 == 0 {
            let footstep_pos = cube_transform.translation - cube_transform.forward() * 0.5;
            let right_offset = cube_transform.right() * 0.3; // Offset for right foot
            let left_offset = cube_transform.right() * -0.3; // Offset for left foot
            
            // Alternate which foot is forward to create walking gait
            let step_phase = (current_time * 2.0) as i32 % 2;
            let forward_offset = if step_phase == 0 { 0.2 } else { -0.2 };
            
            // Right foot
            let right_pos = Vec3::new(footstep_pos.x, 0.05, footstep_pos.z) + right_offset + cube_transform.forward() * forward_offset;
            commands.spawn((
                Mesh3d(meshes.add(Circle::new(0.1))),
                MeshMaterial3d(materials.add(StandardMaterial {
                    base_color: Color::srgb(0.0, 0.0, 0.0), // Black
                    emissive: Color::srgb(0.0, 0.0, 0.0).into(), // Emissive black
                    alpha_mode: AlphaMode::Blend,
                    ..default()
                })),
                Transform::from_translation(right_pos)
                    .with_rotation(Quat::from_rotation_x(-std::f32::consts::FRAC_PI_2)),
                GlobalTransform::default(),
                Visibility::default(),
                InheritedVisibility::default(),
                Footstep {
                    lifetime: 0.0,
                    max_lifetime: 3.0,
                },
            ));
            
            // Left foot
            let left_pos = Vec3::new(footstep_pos.x, 0.05, footstep_pos.z) + left_offset + cube_transform.forward() * (-forward_offset);
            commands.spawn((
                Mesh3d(meshes.add(Circle::new(0.1))),
                MeshMaterial3d(materials.add(StandardMaterial {
                    base_color: Color::srgb(0.0, 0.0, 0.0), // Black
                    emissive: Color::srgb(0.0, 0.0, 0.0).into(), // Emissive black
                    alpha_mode: AlphaMode::Blend,
                    ..default()
                })),
                Transform::from_translation(left_pos)
                    .with_rotation(Quat::from_rotation_x(-std::f32::consts::FRAC_PI_2)),
                GlobalTransform::default(),
                Visibility::default(),
                InheritedVisibility::default(),
                Footstep {
                    lifetime: 0.0,
                    max_lifetime: 3.0,
                },
            ));
        }
    }
}

/// System to update and fade out footsteps
fn update_footsteps(
    time: Res<Time>,
    mut footsteps: Query<(Entity, &mut Footstep, &mut Transform)>,
    mut commands: Commands,
) {
    let dt = time.delta_secs();
    
    for (entity, mut footstep, mut transform) in footsteps.iter_mut() {
        footstep.lifetime += dt;
        
        // Calculate fade based on lifetime
        let fade = 1.0 - (footstep.lifetime / footstep.max_lifetime);
        
        if footstep.lifetime >= footstep.max_lifetime {
            commands.entity(entity).despawn();
        } else {
            // Slightly shrink the footstep as it fades
            let scale = 0.3 + fade * 0.7;
            transform.scale = Vec3::splat(scale);
        }
    }
}

// Removed GLB loading system for better performance

/// System to draw wireframe around the red cube
fn draw_wireframe(
    mut gizmos: Gizmos,
    cube_query: Query<&Transform, With<CubeController>>,
) {
    for transform in cube_query.iter() {
        // Draw wireframe cube around the red cube
        gizmos.cuboid(
            *transform,
            Color::srgb(0.7, 0.0, 0.0), // Blood red wireframe
        );
    }
}

/// System to slowly rotate the radar on a fixed axis
fn rotate_radar(
    time: Res<Time>,
    mut radar_query: Query<&mut Transform, With<RotatingRadar>>,
) {
    for mut transform in radar_query.iter_mut() {
        // Rotate slowly around Y axis (fixed axis rotation, not circular movement)
        transform.rotate_y(time.delta_secs() * 0.2); // Much slower rotation: 0.2 radians per second
    }
}

fn spawn_spheres(
    time: Res<Time>,
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut spawner_query: Query<&mut SphereSpawner>,
    sphere_query: Query<Entity, With<ChasingSphere>>,
) {
    for mut spawner in spawner_query.iter_mut() {
        if spawner.timer.tick(time.delta()).just_finished() {
            // Check if we already have 10 spheres
            if sphere_query.iter().count() >= 10 {
                continue; // Don't spawn more spheres
            }
            // Spawn a new UGV at a random location on the ground
            let angle = pseudo_random(time.elapsed_secs() * 1000.0) * 2.0 * std::f32::consts::PI;
            let radius = pseudo_random(time.elapsed_secs() * 1001.0) * 50.0 + 30.0; // 30-80 units away
            let x = angle.cos() * radius;
            let z = angle.sin() * radius;
            let y = 0.0; // Ground level
            
            println!("Spawning UGV at position: ({}, {}, {})", x, y, z);
            commands.spawn((
                SceneRoot(asset_server.load("models/antagonists/ugv/ugv.gltf#Scene0")),
                Transform::from_xyz(x, y, z)
                    .with_scale(Vec3::splat(0.25)), // Make UGV 2x smaller (0.5 -> 0.25)
                GlobalTransform::default(),
                Visibility::default(),
                InheritedVisibility::default(),
                ChasingSphere {
                    speed: 3.0, // 3 units per second
                    last_line_of_sight: false,
                },
            ));
        }
    }
}

fn check_line_of_sight(
    from: Vec3,
    to: Vec3,
    obstacle_query: &Query<(&Transform, &ObstacleBlocker), (Without<CubeController>, Without<ChasingSphere>)>,
) -> bool {
    let direction = (to - from).normalize();
    let distance = from.distance(to);
    
    // For now, let's simplify and just check if there are any obstacles at all
    // If there are no obstacles, always return true
    if obstacle_query.iter().count() == 0 {
        return true;
    }
    
    // Sample points along the ray for collision detection
    let num_samples = (distance / 2.0) as usize + 1; // Sample every 2 units
    
    for i in 0..num_samples {
        let t = (i as f32) / (num_samples as f32 - 1.0);
        let sample_point = from + direction * (distance * t);
        
        // Check all obstacles
        for (obstacle_transform, obstacle_blocker) in obstacle_query.iter() {
            let obstacle_pos = obstacle_transform.translation;
            let obstacle_scale = obstacle_transform.scale;
            
            // Use the stored collision box dimensions
            let half_size = obstacle_blocker.half_size * obstacle_scale;
            let min = obstacle_pos - half_size;
            let max = obstacle_pos + half_size;
            
            // Check if the sample point is inside the obstacle's collision box
            if sample_point.x >= min.x && sample_point.x <= max.x &&
               sample_point.y >= min.y && sample_point.y <= max.y &&
               sample_point.z >= min.z && sample_point.z <= max.z {
                return false; // Line of sight blocked
            }
        }
    }
    
    true // No obstacles found
}

fn chase_cube(
    time: Res<Time>,
    mut sphere_query: Query<(&mut Transform, &mut ChasingSphere)>,
    cube_query: Query<&Transform, (With<CubeController>, Without<ChasingSphere>)>,
    obstacle_query: Query<(&Transform, &ObstacleBlocker), (Without<CubeController>, Without<ChasingSphere>)>,
) {
    if let Ok(cube_transform) = cube_query.single() {
        let sphere_count = sphere_query.iter().count();
        if sphere_count > 0 {
            println!("Chase cube: {} UGV's found, cube at {:?}", sphere_count, cube_transform.translation);
        }
        
        for (mut sphere_transform, mut sphere) in sphere_query.iter_mut() {
            let cube_pos = cube_transform.translation;
            let sphere_pos = sphere_transform.translation;
            let direction = (cube_pos - sphere_pos).normalize();
            let distance = cube_pos.distance(sphere_pos);
            
            // Check if within range
            if distance < 100.0 {
                // Temporarily disable line of sight check to test basic chasing
                let has_line_of_sight = true; // Always true for now
                println!("UGV at {:?}, distance: {:.2}, line of sight: {}", sphere_pos, distance, has_line_of_sight);
                
                if has_line_of_sight {
                    // Chase the cube
                    let move_distance = sphere.speed * time.delta_secs();
                    if distance > move_distance {
                        let new_pos = sphere_transform.translation + direction * move_distance;
                        sphere_transform.translation = new_pos;
                    } else {
                        // Close enough, move directly to cube
                        sphere_transform.translation = cube_pos;
                    }
                    
                    // Rotate UGV to face the cube
                    let look_direction = direction;
                    let rotation = Quat::from_rotation_y(look_direction.x.atan2(look_direction.z));
                    sphere_transform.rotation = rotation;
                    
                    sphere.last_line_of_sight = true;
                } else {
                    sphere.last_line_of_sight = false;
                }
            } else {
                sphere.last_line_of_sight = false;
            }
        }
    }
}

fn draw_line_of_sight(
    mut gizmos: Gizmos,
    sphere_query: Query<(&Transform, &ChasingSphere)>,
    cube_query: Query<&Transform, (With<CubeController>, Without<ChasingSphere>)>,
    obstacle_query: Query<(&Transform, &ObstacleBlocker), (Without<CubeController>, Without<ChasingSphere>)>,
) {
    if let Ok(cube_transform) = cube_query.single() {
        for (sphere_transform, _sphere) in sphere_query.iter() {
            let cube_pos = cube_transform.translation;
            let sphere_pos = sphere_transform.translation;
            let distance = cube_pos.distance(sphere_pos);
            
            if distance < 100.0 {
                let has_line_of_sight = check_line_of_sight(sphere_pos, cube_pos, &obstacle_query);
                
                if has_line_of_sight {
                    // Draw a bright red line when line of sight is clear
                    gizmos.line(
                        sphere_transform.translation,
                        cube_transform.translation,
                        Color::srgb(1.0, 0.0, 0.0), // Bright red
                    );
                } else {
                    // Draw a dim gray line when line of sight is blocked
                    gizmos.line(
                        sphere_transform.translation,
                        cube_transform.translation,
                        Color::srgb(0.3, 0.3, 0.3), // Dim gray
                    );
                }
            }
        }
    }
}

fn despawn_spheres(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    sphere_query: Query<(Entity, &Transform), With<ChasingSphere>>,
    mut cube_query: Query<(&Transform, &mut Health), (With<CubeController>, Without<ChasingSphere>)>,
) {
    for (cube_transform, mut cube_health) in cube_query.iter_mut() {
        let cube_pos = cube_transform.translation;
        
        for (sphere_entity, sphere_transform) in sphere_query.iter() {
            let sphere_pos = sphere_transform.translation;
            let distance = cube_pos.distance(sphere_pos);
            
            // If sphere is close enough to the cube (intersecting)
            if distance < 1.5 { // 1.5 units threshold for intersection
                // Damage the cube
                cube_health.current = (cube_health.current - 10.0).max(0.0);
                
                // Create smoke particles at the sphere's position
                for i in 0..8 {
                    let angle = (i as f32) * 2.0 * std::f32::consts::PI / 8.0;
                    let offset_x = angle.cos() * 0.5;
                    let offset_z = angle.sin() * 0.5;
                    
                    commands.spawn((
                        Mesh3d(meshes.add(Sphere::new(0.1))), // Small smoke particle
                        MeshMaterial3d(materials.add(StandardMaterial {
                            base_color: Color::srgb(0.3, 0.3, 0.3), // Dark gray smoke
                            emissive: Color::srgb(0.1, 0.1, 0.1).into(), // Slightly emissive
                            alpha_mode: AlphaMode::Blend,
                            ..default()
                        })),
                        Transform::from_xyz(
                            sphere_pos.x + offset_x,
                            sphere_pos.y,
                            sphere_pos.z + offset_z,
                        ),
                        GlobalTransform::default(),
                        Visibility::default(),
                        InheritedVisibility::default(),
                        SmokeParticle {
                            lifetime: 2.0, // 2 seconds lifetime
                            max_lifetime: 2.0,
                            velocity: Vec3::new(
                                offset_x * 0.5, // Horizontal drift
                                1.0, // Rising velocity
                                offset_z * 0.5, // Horizontal drift
                            ),
                        },
                    ));
                }
                
                // Despawn the sphere
                commands.entity(sphere_entity).despawn();
            }
        }
    }
}

fn update_smoke(
    time: Res<Time>,
    mut commands: Commands,
    mut smoke_query: Query<(Entity, &mut Transform, &mut SmokeParticle)>,
) {
    for (smoke_entity, mut transform, mut smoke) in smoke_query.iter_mut() {
        // Update lifetime
        smoke.lifetime -= time.delta_secs();
        
        if smoke.lifetime <= 0.0 {
            // Despawn expired smoke particle
            commands.entity(smoke_entity).despawn();
        } else {
            // Move smoke particle
            transform.translation += smoke.velocity * time.delta_secs();
            
            // Fade out over time
            let fade = smoke.lifetime / smoke.max_lifetime;
            let scale = 0.1 + fade * 0.9; // Scale from 0.1 to 1.0
            transform.scale = Vec3::splat(scale);
        }
    }
}

#[cfg(target_arch = "wasm32")]
fn update_health(
    health_query: Query<&Health, (With<CubeController>, Changed<Health>)>,
) {
    for health in health_query.iter() {
        // Call JavaScript function to update the DOM health bar
        if let Some(window) = web_sys::window() {
            if let Ok(update_fn) = js_sys::Reflect::get(&window, &"updateHealth".into()) {
                if let Ok(update_fn) = update_fn.dyn_into::<js_sys::Function>() {
                    let _ = update_fn.call1(&window, &wasm_bindgen::JsValue::from_f64(health.current as f64));
                }
            }
        }
    }
}

#[cfg(not(target_arch = "wasm32"))]
fn update_health(
    _health_query: Query<&Health, (With<CubeController>, Changed<Health>)>,
) {
    // No-op for non-WASM targets
}











