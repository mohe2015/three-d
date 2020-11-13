
use three_d::*;

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let screenshot_path = if args.len() > 1 { Some(args[1].clone()) } else {None};

    let mut window = Window::new_default("Texture").unwrap();
    let (width, height) = window.framebuffer_size();
    let gl = window.gl();

    // Renderer
    let mut renderer = DeferredPipeline::new(&gl).unwrap();
    let mut camera = Camera::new_perspective(&gl, vec3(4.0, 1.5, 4.0), vec3(0.0, 1.0, 0.0), vec3(0.0, 1.0, 0.0),
                                                degrees(45.0), width as f32 / height as f32, 0.1, 1000.0);

    let mut loader = Loader::new();
    loader.start_loading("examples/assets/models/penguin.3d");
    loader.start_loading("examples/assets/textures/test_texture.jpg");
    loader.wait_all(move |loaded| {
        let box_mesh = tri_mesh::MeshBuilder::new().unconnected_cube().build().unwrap();
        let texture = texture::Texture2D::new_from_bytes(&gl, Interpolation::Linear, Interpolation::Linear, Some(Interpolation::Linear),
                           Wrapping::ClampToEdge, Wrapping::ClampToEdge, loaded.get("examples/assets/textures/test_texture.jpg").unwrap().as_ref().unwrap()).unwrap();
        let box_mesh = TexturedMesh::new(&gl, &box_mesh.indices_buffer(), &box_mesh.positions_buffer_f32(), &box_mesh.normals_buffer_f32(), &[], texture).unwrap();

        let texture3d = TextureCubeMap::new_from_bytes(&gl, Interpolation::Linear, Interpolation::Linear, None, Wrapping::ClampToEdge, Wrapping::ClampToEdge, Wrapping::ClampToEdge,
                                                           include_bytes!("../assets/textures/skybox_evening/back.jpg"),
                                                           include_bytes!("../assets/textures/skybox_evening/front.jpg"),
                                                           include_bytes!("../assets/textures/skybox_evening/top.jpg"),
                                                           include_bytes!("../assets/textures/skybox_evening/left.jpg"),
                                                           include_bytes!("../assets/textures/skybox_evening/right.jpg")).unwrap();
        let skybox = objects::Skybox::new(&gl, texture3d);

        let penguin_texture = texture::Texture2D::new_from_bytes(&gl, Interpolation::Linear, Interpolation::Linear, Some(Interpolation::Linear),
                           Wrapping::ClampToEdge, Wrapping::ClampToEdge, include_bytes!("../assets/textures/penguin.png")).unwrap();
        let penguin = TexturedMesh::from_cpu_mesh(&gl, &ThreeD::parse(loaded.get("examples/assets/models/penguin.3d").unwrap().as_ref().unwrap()).unwrap(), penguin_texture).unwrap();

        let ambient_light = AmbientLight::new(&gl, 0.4, &vec3(1.0, 1.0, 1.0)).unwrap();
        let directional_light = DirectionalLight::new(&gl, 1.0, &vec3(1.0, 1.0, 1.0), &vec3(0.0, -1.0, -1.0)).unwrap();

        // main loop
        let mut rotating = false;
        window.render_loop(move |frame_input|
        {
            camera.set_size(frame_input.screen_width as f32, frame_input.screen_height as f32);

            for event in frame_input.events.iter() {
                match event {
                    Event::MouseClick {state, button, ..} => {
                        rotating = *button == MouseButton::Left && *state == State::Pressed;
                    },
                    Event::MouseMotion {delta} => {
                        if rotating {
                            camera.rotate(delta.0 as f32, delta.1 as f32);
                        }
                    },
                    Event::MouseWheel {delta} => {
                        camera.zoom(*delta as f32);
                    },
                    Event::Key { state, kind } => {
                        if kind == "R" && *state == State::Pressed
                        {
                            renderer.next_debug_type();
                            println!("{:?}", renderer.debug_type());
                        }
                    }
                }
            }

            // draw
            // Geometry pass
            renderer.geometry_pass(width, height, &|| {
                let mut transformation = Mat4::identity();
                box_mesh.render(&transformation, &camera);
                transformation = Mat4::from_translation(vec3(0.0, 1.0, 0.0));
                penguin.render(&transformation, &camera);
            }).unwrap();

            Screen::write(&gl, 0, 0, width, height, Some(&vec4(0.8, 0.0, 0.0, 1.0)), None, &|| {
                skybox.render(&camera).unwrap();
                renderer.light_pass(&camera, Some(&ambient_light), &[&directional_light], &[], &[]).unwrap();
            }).unwrap();

            if let Some(ref path) = screenshot_path {
                #[cfg(target_arch = "x86_64")]
                Screen::save_color(path, &gl, 0, 0, width, height).unwrap();
                std::process::exit(1);
            }
        }).unwrap();
    });


}