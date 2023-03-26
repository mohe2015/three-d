use three_d::*;
use winit::event_loop::EventLoop;
use winit::event_loop::EventLoopWindowTarget;

pub fn main() {
    let event_loop = EventLoop::new();

    let mut event_loop_1 = create_window(&event_loop);
    //let mut event_loop_2 = create_window(&event_loop);

    event_loop.run(move |event, target, control_flow| {
        event_loop_1(&event, target, control_flow);
        //event_loop_2(&event, target, control_flow);
    })
}

fn create_window<T: 'static + Clone>(event_loop: &EventLoop<T>) -> impl FnMut(&winit::event::Event<()>, &EventLoopWindowTarget<()>, &mut winit::event_loop::ControlFlow) {
    // Create a window (a canvas on web)
    let window = Window::from_event_loop(WindowSettings {
        title: "Triangle!".to_string(),
        max_size: Some((1280, 720)),
        ..Default::default()
    }, event_loop)
    .unwrap();

    // Get the graphics context from the window
    let context = window.gl();

    // Create a camera
    let mut camera = Camera::new_perspective(
        window.viewport(),
        vec3(0.0, 0.0, 2.0),
        vec3(0.0, 0.0, 0.0),
        vec3(0.0, 1.0, 0.0),
        degrees(45.0),
        0.1,
        10.0,
    );

    // Create a CPU-side mesh consisting of a single colored triangle
    let positions = vec![
        vec3(0.5, -0.5, 0.0),  // bottom right
        vec3(-0.5, -0.5, 0.0), // bottom left
        vec3(0.0, 0.5, 0.0),   // top
    ];
    let colors = vec![
        Color::new(255, 0, 0, 255), // bottom right
        Color::new(0, 255, 0, 255), // bottom left
        Color::new(0, 0, 255, 255), // top
    ];
    let cpu_mesh = CpuMesh {
        positions: Positions::F32(positions),
        colors: Some(colors),
        ..Default::default()
    };

    // Construct a model, with a default color material, thereby transferring the mesh data to the GPU
    let mut model = Gm::new(Mesh::new(&context, &cpu_mesh), ColorMaterial::default());

    // Add an animation to the triangle.
    model.set_animation(|time| Mat4::from_angle_y(radians(time * 0.005)));

    // Start the main render loop
    window.get_render_loop_impl(move |frame_input: FrameInput<()>| // Begin a new frame with an updated frame input
    {
        // Ensure the viewport matches the current window viewport which changes if the window is resized
        camera.set_viewport(frame_input.viewport);

        // Update the animation of the triangle
        model.animate(frame_input.accumulated_time as f32);

        // Get the screen render target to be able to render something on the screen
        frame_input.screen()
            // Clear the color and depth of the screen render target
            .clear(ClearState::color_and_depth(0.8, 0.8, 0.8, 1.0, 1.0))
            // Render the triangle with the color material which uses the per vertex colors defined at construction
            .render(
                &camera, &model, &[]
            );

        // Returns default frame output to end the frame
        FrameOutput::default()
    },
    )
}