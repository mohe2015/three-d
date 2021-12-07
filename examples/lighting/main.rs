use three_d::*;

#[derive(Debug, Eq, PartialEq)]
enum Pipeline {
    Forward,
    Deferred,
}

fn main() {
    let args: Vec<String> = std::env::args().collect();

    let window = Window::new(WindowSettings {
        title: "Lighting!".to_string(),
        min_size: (512, 512),
        max_size: Some((1280, 720)),
        ..Default::default()
    })
    .unwrap();
    let context = window.gl().unwrap();

    let forward_pipeline = ForwardPipeline::new(&context).unwrap();
    let mut deferred_pipeline = DeferredPipeline::new(&context).unwrap();
    let mut camera = Camera::new_perspective(
        &context,
        window.viewport().unwrap(),
        vec3(2.0, 2.0, 5.0),
        vec3(0.0, 0.0, 0.0),
        vec3(0.0, 1.0, 0.0),
        degrees(45.0),
        0.1,
        30.0,
    )
    .unwrap();
    let mut control = OrbitControl::new(*camera.target(), 1.0, 100.0);
    let mut gui = three_d::GUI::new(&context).unwrap();

    let model = Loading::new(
        &context,
        &["examples/assets/gltf/DamagedHelmet.glb"],
        move |context, mut loaded| {
            let (mut cpu_meshes, cpu_materials) = loaded.gltf("DamagedHelmet.glb").unwrap();
            let mut material = PhysicalMaterial::new(&context, &cpu_materials[0]).unwrap();
            material.opaque_render_states.cull = Cull::Back;
            cpu_meshes[0].compute_tangents().unwrap();
            let mut model =
                Model::new_with_material(&context, &cpu_meshes[0], material.clone()).unwrap();
            model.set_transformation(Mat4::from_angle_x(degrees(90.0)));
            Ok(model)
        },
    );
    let mut plane = Model::new_with_material(
        &context,
        &CPUMesh::square(),
        PhysicalMaterial {
            albedo: Color::new_opaque(128, 200, 70),
            ..Default::default()
        },
    )
    .unwrap();
    plane.set_transformation(
        Mat4::from_translation(vec3(0.0, -1.0, 0.0))
            * Mat4::from_scale(10.0)
            * Mat4::from_angle_x(degrees(-90.0)),
    );

    let mut lights = Lights {
        ambient: Some(AmbientLight {
            color: Color::WHITE,
            intensity: 0.2,
        }),
        directional: vec![
            DirectionalLight::new(&context, 1.0, Color::RED, &vec3(0.0, -1.0, 0.0)).unwrap(),
            DirectionalLight::new(&context, 1.0, Color::GREEN, &vec3(0.0, -1.0, 0.0)).unwrap(),
        ],
        spot: vec![SpotLight::new(
            &context,
            2.0,
            Color::BLUE,
            &vec3(0.0, 0.0, 0.0),
            &vec3(0.0, -1.0, 0.0),
            degrees(25.0),
            0.1,
            0.001,
            0.0001,
        )
        .unwrap()],
        point: vec![
            PointLight::new(
                &context,
                1.0,
                Color::GREEN,
                &vec3(0.0, 0.0, 0.0),
                0.5,
                0.05,
                0.005,
            )
            .unwrap(),
            PointLight::new(
                &context,
                1.0,
                Color::RED,
                &vec3(0.0, 0.0, 0.0),
                0.5,
                0.05,
                0.005,
            )
            .unwrap(),
        ],
        ..Default::default()
    };

    // main loop
    let mut shadows_enabled = true;
    let mut directional_intensity = lights.directional[0].intensity();
    let mut spot_intensity = lights.spot[0].intensity();
    let mut point_intensity = lights.point[0].intensity();

    let mut current_pipeline = Pipeline::Forward;

    window
        .render_loop(move |mut frame_input| {
            let mut change = frame_input.first_frame;
            let mut panel_width = frame_input.viewport.width;
            change |= gui
                .update(&mut frame_input, |gui_context| {
                    use three_d::egui::*;
                    SidePanel::left("side_panel").show(gui_context, |ui| {
                        ui.heading("Debug Panel");

                        ui.label("Surface parameters");
                        if let Some(Ok(ref mut model)) = *model.borrow_mut() {
                            ui.add(
                                Slider::new(&mut model.material.metallic, 0.0..=1.0)
                                    .text("Model Metallic"),
                            );
                            ui.add(
                                Slider::new(&mut model.material.roughness, 0.0..=1.0)
                                    .text("Model Roughness"),
                            );
                            ui.add(
                                Slider::new(&mut model.material.albedo.a, 0..=255)
                                    .text("Model opacity"),
                            );
                        }
                        ui.add(
                            Slider::new(&mut plane.material.metallic, 0.0..=1.0)
                                .text("Plane Metallic"),
                        );
                        ui.add(
                            Slider::new(&mut plane.material.roughness, 0.0..=1.0)
                                .text("Plane Roughness"),
                        );

                        ui.label("Light options");
                        ui.add(
                            Slider::new(&mut lights.ambient.as_mut().unwrap().intensity, 0.0..=1.0)
                                .text("Ambient intensity"),
                        );
                        ui.add(
                            Slider::new(&mut directional_intensity, 0.0..=1.0)
                                .text("Directional intensity"),
                        );
                        lights.directional[0].set_intensity(directional_intensity);
                        lights.directional[1].set_intensity(directional_intensity);
                        ui.add(Slider::new(&mut spot_intensity, 0.0..=1.0).text("Spot intensity"));
                        lights.spot[0].set_intensity(spot_intensity);
                        ui.add(
                            Slider::new(&mut point_intensity, 0.0..=1.0).text("Point intensity"),
                        );
                        lights.point[0].set_intensity(point_intensity);
                        lights.point[1].set_intensity(point_intensity);
                        if ui.checkbox(&mut shadows_enabled, "Shadows").clicked() {
                            if !shadows_enabled {
                                lights.spot[0].clear_shadow_map();
                                lights.directional[0].clear_shadow_map();
                                lights.directional[1].clear_shadow_map();
                            }
                        }

                        ui.label("Lighting model");
                        ui.radio_value(&mut lights.lighting_model, LightingModel::Phong, "Phong");
                        ui.radio_value(&mut lights.lighting_model, LightingModel::Blinn, "Blinn");
                        ui.radio_value(
                            &mut lights.lighting_model,
                            LightingModel::Cook(
                                NormalDistributionFunction::Blinn,
                                GeometryFunction::SmithSchlickGGX,
                            ),
                            "Cook (Blinn)",
                        );
                        ui.radio_value(
                            &mut lights.lighting_model,
                            LightingModel::Cook(
                                NormalDistributionFunction::Beckmann,
                                GeometryFunction::SmithSchlickGGX,
                            ),
                            "Cook (Beckmann)",
                        );
                        ui.radio_value(
                            &mut lights.lighting_model,
                            LightingModel::Cook(
                                NormalDistributionFunction::TrowbridgeReitzGGX,
                                GeometryFunction::SmithSchlickGGX,
                            ),
                            "Cook (Trowbridge-Reitz GGX)",
                        );

                        ui.label("Pipeline");
                        ui.radio_value(&mut current_pipeline, Pipeline::Forward, "Forward");
                        ui.radio_value(&mut current_pipeline, Pipeline::Deferred, "Deferred");
                        ui.label("Debug options");
                        ui.radio_value(&mut deferred_pipeline.debug_type, DebugType::NONE, "None");
                        ui.radio_value(
                            &mut deferred_pipeline.debug_type,
                            DebugType::POSITION,
                            "Position",
                        );
                        ui.radio_value(
                            &mut deferred_pipeline.debug_type,
                            DebugType::NORMAL,
                            "Normal",
                        );
                        ui.radio_value(
                            &mut deferred_pipeline.debug_type,
                            DebugType::COLOR,
                            "Color",
                        );
                        ui.radio_value(&mut deferred_pipeline.debug_type, DebugType::UV, "UV");
                        ui.radio_value(
                            &mut deferred_pipeline.debug_type,
                            DebugType::DEPTH,
                            "Depth",
                        );
                        ui.radio_value(&mut deferred_pipeline.debug_type, DebugType::ORM, "ORM");
                    });
                    panel_width = gui_context.used_size().x as u32;
                })
                .unwrap();

            let viewport = Viewport {
                x: panel_width as i32,
                y: 0,
                width: frame_input.viewport.width - panel_width,
                height: frame_input.viewport.height,
            };
            change |= camera.set_viewport(viewport).unwrap();
            change |= control
                .handle_events(&mut camera, &mut frame_input.events)
                .unwrap();

            let time = 0.001 * frame_input.accumulated_time;
            let c = time.cos() as f32;
            let s = time.sin() as f32;
            lights.directional[0].set_direction(&vec3(-1.0 - c, -1.0, 1.0 + s));
            lights.directional[1].set_direction(&vec3(1.0 + c, -1.0, -1.0 - s));
            lights.spot[0].set_position(&vec3(3.0 + c, 5.0 + s, 3.0 - s));
            lights.spot[0].set_direction(&-vec3(3.0 + c, 5.0 + s, 3.0 - s));
            lights.point[0].set_position(&vec3(-5.0 * c, 5.0, -5.0 * s));
            lights.point[1].set_position(&vec3(5.0 * c, 5.0, 5.0 * s));

            // Draw
            if let Some(Ok(ref model)) = *model.borrow() {
                if shadows_enabled {
                    lights.directional[0]
                        .generate_shadow_map(4.0, 1024, 1024, &[model, &plane])
                        .unwrap();
                    lights.directional[1]
                        .generate_shadow_map(4.0, 1024, 1024, &[model, &plane])
                        .unwrap();
                    lights.spot[0]
                        .generate_shadow_map(1024, &[model, &plane])
                        .unwrap();
                }

                // Geometry pass
                if change && current_pipeline == Pipeline::Deferred {
                    deferred_pipeline
                        .render_pass(
                            &camera,
                            &[
                                (
                                    model,
                                    DeferredPhysicalMaterial::from_physical_material(
                                        &model.material,
                                    ),
                                ),
                                (
                                    &plane,
                                    DeferredPhysicalMaterial::from_physical_material(
                                        &plane.material,
                                    ),
                                ),
                            ],
                        )
                        .unwrap();
                }

                // Light pass
                Screen::write(&context, ClearState::default(), || {
                    match current_pipeline {
                        Pipeline::Forward => {
                            match deferred_pipeline.debug_type {
                                DebugType::NORMAL => {
                                    plane.render_with_material(
                                        &NormalMaterial::from_physical_material(&plane.material),
                                        &camera,
                                        &lights,
                                    )?;
                                    model.render_with_material(
                                        &NormalMaterial::from_physical_material(&model.material),
                                        &camera,
                                        &lights,
                                    )?;
                                }
                                DebugType::DEPTH => {
                                    let depth_material = DepthMaterial::default();
                                    plane.render_with_material(
                                        &depth_material,
                                        &camera,
                                        &lights,
                                    )?;
                                    model.render_with_material(
                                        &depth_material,
                                        &camera,
                                        &lights,
                                    )?;
                                }
                                DebugType::ORM => {
                                    plane.render_with_material(
                                        &ORMMaterial::from_physical_material(&plane.material),
                                        &camera,
                                        &lights,
                                    )?;
                                    model.render_with_material(
                                        &ORMMaterial::from_physical_material(&model.material),
                                        &camera,
                                        &lights,
                                    )?;
                                }
                                DebugType::POSITION => {
                                    let position_material = PositionMaterial::default();
                                    plane.render_with_material(
                                        &position_material,
                                        &camera,
                                        &lights,
                                    )?;
                                    model.render_with_material(
                                        &position_material,
                                        &camera,
                                        &lights,
                                    )?;
                                }
                                DebugType::UV => {
                                    let uv_material = UVMaterial::default();
                                    plane.render_with_material(&uv_material, &camera, &lights)?;
                                    model.render_with_material(&uv_material, &camera, &lights)?;
                                }
                                DebugType::COLOR => {
                                    plane.render_with_material(
                                        &ColorMaterial::from_physical_material(&plane.material),
                                        &camera,
                                        &lights,
                                    )?;
                                    model.render_with_material(
                                        &ColorMaterial::from_physical_material(&model.material),
                                        &camera,
                                        &lights,
                                    )?;
                                }
                                DebugType::NONE => forward_pipeline.render_pass(
                                    &camera,
                                    &[&plane, &model],
                                    &lights,
                                )?,
                            };
                        }
                        Pipeline::Deferred => {
                            deferred_pipeline.lighting_pass(&camera, &lights)?;
                        }
                    }
                    gui.render()?;
                    Ok(())
                })
                .unwrap();
            }

            if args.len() > 1 {
                // To automatically generate screenshots of the examples, can safely be ignored.
                FrameOutput {
                    screenshot: Some(args[1].clone().into()),
                    exit: true,
                    ..Default::default()
                }
            } else {
                FrameOutput::default()
            }
        })
        .unwrap();
}
