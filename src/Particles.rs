use std::time::{Duration, Instant};
use std::clone::Clone;
use std::sync::Arc;
use std::sync::{Mutex};
use std::sync::atomic::{Ordering::SeqCst, AtomicU32};



#[derive(Debug,Copy,Clone)]
pub struct Particle {
    pub velocity_x: f64,
    pub velocity_y: f64,
    pub speed:f64,
    pub x: f64,
    pub y: f64,
    pub dist: f64,
}

impl Particle {
    pub fn new(x: f64, y: f64) -> Particle {
        Particle {
            velocity_x: 0.0,
            velocity_y:0.0,
            speed:0.0,
            x: x,
            y: y,
            dist:0.0,
        }
    }

    pub fn collide(&mut self, particle2: Particle) -> f64 {
        let dx = self.x - particle2.x;
        let dy = self.y - particle2.y;

        let mut distance = (dx.powf(2.0)) + (dy.powf(2.0));
        distance = distance.sqrt();
        
        return distance;  
    }

    pub fn print(&mut self, id:usize) {
        print!("\nParticle {}:(x: {}, y: {})",id,self.x,self.y);
    }
}

#[derive(Debug,Clone)]
pub struct ParticleSystem {
    pub particle_list: Vec<Particle>, 
    pub collisions: Arc<AtomicU32>
}

impl ParticleSystem {
   pub fn new() -> ParticleSystem {
        ParticleSystem{
            particle_list: vec!(),
            collisions: Arc::new(AtomicU32::new(0))
        }
   }
   pub fn init(&mut self,size: i32) {
        for _id in 0..size {
            let random_value_x = ((rand::random::<f64>() * 2.0) - 1.0) * 50.0;
            let random_value_y = ((rand::random::<f64>() * 2.0) - 1.0) * 50.0;
            let p = Particle::new(random_value_x, random_value_y);
            self.particle_list.push(p);
        }
   }
}








