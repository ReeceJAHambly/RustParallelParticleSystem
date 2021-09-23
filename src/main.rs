#[macro_use]
extern crate glium;
extern crate rand;

mod Particles;
use Particles::Particle;
use Particles::ParticleSystem;
use std::sync::Arc;
use std::sync::atomic::{Ordering::SeqCst, AtomicU32};
use std::time::{Duration,SystemTime};

fn main() {
    #[allow(unused_imports)]
    use glium::{glutin, Surface};
    //use rand;
    let event_loop = glutin::event_loop::EventLoop::new();
    let wb = glutin::window::WindowBuilder::new();
    let cb = glutin::ContextBuilder::new();
    let display = glium::Display::new(wb, cb, &event_loop).unwrap();

    #[derive(Copy, Clone)]
    struct Vertex {
        position: [f32; 3],
        colour: [f32; 3],
    }

    implement_vertex!(Vertex, position,colour);
   
    const NUM_OF_THREADS_MOVEMENT: usize = 5;
   
    let mut NUM_OF_PARTICLES: usize = 100;  
    let originalParitcles = NUM_OF_PARTICLES;
    let mut system_particles = ParticleSystem::new();
    system_particles.init(NUM_OF_PARTICLES as i32);

    //let mut collisionCount = 0;

    let mut movement_pool = scoped_threadpool::Pool::new(NUM_OF_PARTICLES as u32/NUM_OF_THREADS_MOVEMENT as u32); 
    let mut gravity_pool = scoped_threadpool::Pool::new(NUM_OF_PARTICLES as u32);
    let mut wind_pool = scoped_threadpool::Pool::new(NUM_OF_PARTICLES as u32); 
    let mut collision_pool = scoped_threadpool::Pool::new(NUM_OF_PARTICLES as u32);     

    //Triangle
    let mut vertex1 = Vertex { position: [-0.50, -0.288,0.0], colour:[1.0,0.0,1.0] };
    let mut vertex2 = Vertex { position: [ 0.00,  0.577,0.0], colour:[1.0,0.0,1.0] };
    let mut vertex3 = Vertex { position: [ 0.50, -0.288,0.0], colour:[1.0,0.0,1.0] };
    let indices = glium::index::NoIndices(glium::index::PrimitiveType::TrianglesList);


    let vertex1_container = Vertex { position: [50.0, 50.0,0.0], colour:[1.0,0.0,0.0] };
    let vertex2_container = Vertex { position: [-50.0,50.0,0.0], colour:[1.0,0.0,0.0]  };
    let vertex3_container = Vertex { position: [-50.0,-51.0,0.0], colour:[1.0,0.0,0.0]  };
    let vertex4_container = Vertex { position: [50.0, -51.0,0.0], colour:[1.0,0.0,0.0]  };
    let shape_container = vec![vertex1_container, vertex2_container, vertex3_container,vertex4_container];

    pub const container_indices: [u16; 5] = [0,1,2,3,0];

    let vertex_buffer_container = glium::VertexBuffer::new(&display, &shape_container).unwrap();
    let indices_container = glium::IndexBuffer::new(&display,glium::index::PrimitiveType::LineStrip,&container_indices).unwrap();


    let vertex_shader_src = r#"
        #version 140

        in vec3 position;
        in vec3 colour;

        out vec3 fblue;

        uniform mat4 matrix;
        uniform mat4 perspective;

        void main() {
            fblue = colour;
            gl_Position = perspective * matrix * vec4(position, 1.0);
        }
    "#;

    let fragment_shader_src = r#"
        #version 140

        out vec4 color;
        in vec3 fblue;

        void main() {
            color = vec4(fblue, 1.0);       
        }
    "#;
    let mut gravity = false;
    let mut wind = false;
    
    let mut speed_render_mode = false; 
    let mut proximity_render_mode = false;
    let mut advanced_render_mode = false;  

    let program = glium::Program::from_source(&display, vertex_shader_src, fragment_shader_src, None).unwrap();
   
    let mut delta_t: f32 = -0.5;
    event_loop.run(move |event, _, control_flow| {
        match event {
            glutin::event::Event::WindowEvent { event, .. } => match event {
                glutin::event::WindowEvent::CloseRequested => {
                    *control_flow = glutin::event_loop::ControlFlow::Exit;
                    return;
                },
                glutin::event::WindowEvent::KeyboardInput { device_id: _, input, is_synthetic: _} => {             
                    if input.virtual_keycode.unwrap() ==  glutin::event::VirtualKeyCode::G && input.state ==  glutin::event::ElementState::Pressed  {
                        gravity = true;
                    }
                    else{
                        gravity = false;
                    }
                    if input.virtual_keycode.unwrap() ==  glutin::event::VirtualKeyCode::W && input.state ==  glutin::event::ElementState::Pressed  {
                        wind = true;
                    }
                    else{
                        wind = false;
                    }

                    if input.virtual_keycode.unwrap() ==  glutin::event::VirtualKeyCode::B {
                        //println! ("Keypressed {:?}", input.virtual_keycode.unwrap());
                        speed_render_mode = true;
                        proximity_render_mode = false;
                        //advanced_render_mode = false;
                    }
                    if input.virtual_keycode.unwrap() ==  glutin::event::VirtualKeyCode::N {
                        //println! ("Keypressed {:?}", input.virtual_keycode.unwrap());
                        proximity_render_mode = true;
                        speed_render_mode = false; 
                        //advanced_render_mode = false; 
                    }
                    if input.virtual_keycode.unwrap() ==  glutin::event::VirtualKeyCode::V {
                        //println! ("Keypressed {:?}", input.virtual_keycode.unwrap());
                        speed_render_mode = false;
                        proximity_render_mode = false; 
                    }
                    if input.virtual_keycode.unwrap() ==  glutin::event::VirtualKeyCode::A {
                        //println! ("Keypressed {:?}", input.virtual_keycode.unwrap());
                        //speed_render_mode = false;
                        //proximity_render_mode = false; 
                        advanced_render_mode = true;
                    }
                },
                _ => return,
            },

            glutin::event::Event::NewEvents(cause) => match cause {
                glutin::event::StartCause::ResumeTimeReached { .. } => (),
                glutin::event::StartCause::Init => (),
                _ => return,
            },
            _ => return,
        }
        

        let next_frame_time = std::time::Instant::now() + std::time::Duration::from_nanos(16_666_667);
        *control_flow = glutin::event_loop::ControlFlow::WaitUntil(next_frame_time);
        
        // Begin render loop

        // Animation counter
        delta_t += 0.005;
        if delta_t > 0.7 {
            delta_t = -1.4;
        }    
       
        let mut startWrite = SystemTime::now();

        movement_pool.scoped(|scope| {      
            for mut slice in system_particles.particle_list.chunks_mut(NUM_OF_PARTICLES/NUM_OF_THREADS_MOVEMENT) {          
                scope.execute(move || thread_main(&mut slice));       
            }       
        }); 
       
        if gravity == true {   
            gravity_pool.scoped(|scope| {      
                for mut slice in system_particles.particle_list.chunks_mut(NUM_OF_PARTICLES) {                        
                        scope.execute(move || thread_gravity(&mut slice));                        
                }  
            });  
        }
     
        if wind == true { 
            wind_pool.scoped(|scope| {      
                for mut slice in system_particles.particle_list.chunks_mut(NUM_OF_PARTICLES) {                          
                            scope.execute(move || thread_wind(&mut slice));                       
                }  
            });
        }
 
        collision_pool.scoped(|scope| {      
            for mut slice in system_particles.particle_list.chunks_mut(50) {   
                let atomic_clone = system_particles.collisions.clone();         
                scope.execute(move || thread_collide(&mut slice,atomic_clone));   
            }  
        });
        if advanced_render_mode == true {
            //println!("{}",system_particles.collisions.load(SeqCst));
            NUM_OF_PARTICLES += system_particles.collisions.load(SeqCst) as usize;
            system_particles.init(system_particles.collisions.load(SeqCst) as i32);
        }

      

        // Create a drawing target
        let mut target = display.draw();

        // Clear the screen to black
        target.clear_color(0.0, 0.0, 0.0, 1.0);

        //calculate for camera
        let (width, height) = target.get_dimensions();
        let aspect_ratio = height as f32 / width as f32;
        let fov: f32 = 3.141592 / 3.0; // Field of view
        let zfar = 1024.0;  // Far clipping plain
        let znear = 0.1; // Near clipping plain
        let f = 1.0 / (fov / 2.0).tan();
        
        
        // Iterate over particles
        for _i in 0 .. NUM_OF_PARTICLES {    
           let x = system_particles.particle_list[_i].x;
           let y = system_particles.particle_list[_i].y;
            
            // Calculate the position of the triangle
            let pos_x : f32 = x as f32; 
            let pos_y : f32 = y as f32; 
            let pos_z : f32 = 10.0;
            // Create a 4x4 matrix to store the position and orientation of the triangle
            let uniforms = uniform! {
                matrix: [
                    [1.0, 0.0, 0.0, 0.0],
                    [0.0, 1.0, 0.0, 0.0],
                    [0.0, 0.0, 1.0, 0.0],
                    [pos_x, pos_y, pos_z, 10.0],],
                perspective: [
                    [f*aspect_ratio, 0.0, 0.0, 0.0],
                    [0.0, f, 0.0, 0.0],
                    [0.0, 0.0, (zfar+znear)/(zfar-znear), 10.0],
                    [0.0, 0.0, -(2.0*zfar*znear)/(zfar-znear), 0.0],             
                ]            
            };
           
            
           // systemParticles.particle_list[_i].calSpeed();
            let mut shape = vec![vertex1, vertex2, vertex3];
            if speed_render_mode == true {             
               
                let speed = system_particles.particle_list[_i].speed;
                if _i < originalParitcles {
                    vertex1.colour = [speed as f32/5.0,0.0,0.0];
                    vertex2.colour = [speed as f32/5.0,0.0,0.0];
                    vertex3.colour = [speed as f32/5.0,0.0,0.0];
                }
                else {
                    vertex1.colour = [0.0,0.0,speed as f32/5.0];
                    vertex2.colour = [0.0,0.0,speed as f32/5.0];
                    vertex3.colour = [0.0,0.0,speed as f32/5.0];
                }
                shape = vec![vertex1, vertex2, vertex3];
            }
            else if proximity_render_mode == true {  
                let mut min = 50.0;   
                for _j in 0..NUM_OF_PARTICLES {
                    let particle = system_particles.particle_list[_j];
                    if _i != _j {                 
                        let distance = system_particles.particle_list[_i].collide(particle);
                        if distance < min {
                            min = distance;
                        }
                        system_particles.particle_list[_i].dist = min;
                    }
                }
                if _i < originalParitcles {
                vertex1.colour = [1.0 - system_particles.particle_list[_i].dist as f32/10.0,0.0,0.0];
                vertex2.colour = [1.0 - system_particles.particle_list[_i].dist as f32/10.0,0.0,0.0];
                vertex3.colour = [1.0 - system_particles.particle_list[_i].dist as f32/10.0,0.0,0.0];
                }
                else 
                {
                    vertex1.colour = [0.0,0.0,1.0 - system_particles.particle_list[_i].dist as f32/10.0];
                    vertex2.colour = [0.0,0.0,1.0 - system_particles.particle_list[_i].dist as f32/10.0];
                    vertex3.colour = [0.0,0.0,1.0 - system_particles.particle_list[_i].dist as f32/10.0];
                }
                shape = vec![vertex1, vertex2, vertex3];
            }
            else {
                if _i < originalParitcles {
                    vertex1.colour = [1.0,0.0,0.0];
                    vertex2.colour = [1.0,0.0,0.0];
                    vertex3.colour = [1.0,0.0,0.0];
                }
                else {
                    vertex1.colour = [0.0,0.0,1.0];
                    vertex2.colour = [0.0,0.0,1.0];
                    vertex3.colour = [0.0,0.0,1.0];
                }
               
                shape = vec![vertex1, vertex2, vertex3];
            }

            let vertex_buffer = glium::VertexBuffer::new(&display, &shape).unwrap();    
            // Draw the triangle
            target.draw(&vertex_buffer, &indices, &program, &uniforms, &Default::default()).unwrap();
        }
        

        let container_x : f32 = 0.0; 
        let container_y : f32 = 0.0;
        let container_z : f32 = 10.0;

        let container_uniforms = uniform! {
            matrix: [
                [1.0, 0.0, 0.0, 0.0],
                [0.0, 1.0, 0.0, 0.0],
                [0.0, 0.0, 1.0, 0.0],
                [container_x, container_y, container_z, 1.0],],
            perspective: [
                [f*aspect_ratio, 0.0, 0.0, 0.0],
                [0.0, f, 0.0, 0.0],
                [0.0, 0.0, (zfar+znear)/(zfar-znear), 10.0],
                [0.0, 0.0, -(2.0*zfar*znear)/(zfar-znear), 0.0],
            ]
        };

        target.draw(&vertex_buffer_container, &indices_container, &program, &container_uniforms, &Default::default()).unwrap();

        // Display the completed drawing
        target.finish().unwrap();
        //let timing = startWrite.elapsed().unwrap().as_micros();
        //println!("{}",timing);
       
        // End render loop
    });
}

pub fn thread_collide(list: &mut [Particle],value: Arc<AtomicU32>) {
     for i in 0..list.len()
     {
         for j in 0..list.len()
         {
             if j != i
             {
                 if list[i].collide(list[j]) < 0.05
                 {              
                        value.store(value.load(SeqCst)+1,SeqCst);              
                 }
             }           
         }      
     }
 }


 pub fn thread_main(list:&mut [Particle]) {
    for i in 0..list.len()
    {          
               
        let random_value_x = (rand::random::<f64>() * 2.0) - 1.0;
        let random_value_y = (rand::random::<f64>() * 2.0) - 1.0;

        if list[i].x > 50.0 {
            list[i].x = -50.0;
        }
        if list[i].x < -50.0 {
            list[i].x = 50.0;
        }
        list[i].x += random_value_x as f64;

        if list[i].y > 50.0 {
            list[i].y = -50.0;
        }
        if list[i].y < -50.0 {
            list[i].y = 50.0;
        }
        list[i].y += random_value_y as f64;   

        list[i].velocity_x = random_value_x;
        list[i].velocity_y = random_value_y;

        list[i].speed = ((random_value_x * random_value_x) + (random_value_y * random_value_y)).sqrt();

        
    }
}

 pub fn thread_wind(list: &mut [Particle]){
    
    //let random_value_x = (rand::random::<f64>() * 2.0) - 1.0;
    let random_value_y = (rand::random::<f64>() * 2.0) - 1.0;
    for i in 0..list.len()
    {      
        if list[i].x > 50.0 {
            list[i].x = -50.0;
        }
        if list[i].x < -50.0 {
            list[i].x = 50.0;
        }

        if list[i].y > 50.0 {
            list[i].y = -50.0;
        }
        if list[i].y < -50.0 {
            list[i].y = 50.0;
        } 
  
        list[i].x += 0.9 as f64;
        list[i].y += 0.5 * random_value_y as f64;

        //list[i].velocity_x += 0.9 as f64;
        //list[i].velocity_y += 0.5 * random_value_y as f64;

        list[i].speed += ((((0.5 * random_value_y as f64) * (0.5 * random_value_y as f64)) + (0.9 as f64 * 0.9 as f64))).sqrt();
        
    }
}

pub fn thread_gravity(list:&mut [Particle]) {
    for i in 0..list.len()
    {      
        if list[i].x > 50.0 {
            list[i].x = -50.0;
        }
        if list[i].x < -50.0 {
            list[i].x = 50.0;
        }

        if list[i].y > 50.0 {
            list[i].y = -50.0;
        }
        if list[i].y < -50.0 {
            list[i].y = 50.0;
        } 
        list[i].y += -0.981 as f64;   
        list[i].speed += (-0.981 as f64 * -0.981 as f64).sqrt();
    }
}
