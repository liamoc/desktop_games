use std::time::{Instant, Duration};
use std::thread;
/// Structure holding the state and timing information of the framerate controller.
pub struct FPSManager {
    framecount : u32,
    rateticks : f32,
	baseticks : u32,
	lastticks : u32,
	rate : u32,
    created : Instant,
}

impl FPSManager {
    const FPS_UPPER_LIMIT : u32 = 200;
    const FPS_LOWER_LIMIT : u32 = 1;
    const FPS_DEFAULT : u32 = 30;
    /// Create the framerate manager.
    fn get_ticks(&self) -> u32 {
        self.created.elapsed().as_millis() as u32
    }
    fn wait_ticks(&self, w: u32) {
        thread::sleep(Duration::from_millis(w as u64))
    }
    pub fn new() -> FPSManager {
        let instant = Instant::now();
        let ticks = instant.elapsed().as_millis() as u32;
        FPSManager {
            framecount: 0,
            rate: Self::FPS_DEFAULT,
            rateticks: (1000.0 / Self::FPS_DEFAULT as f32),
            baseticks: ticks,
            lastticks: ticks,
            created: instant,
        }
    }

    /// Set the framerate in Hz.
    pub fn set_framerate(&mut self, rate: u32) -> Result<(), String> {
        if rate >= Self::FPS_LOWER_LIMIT && rate <= Self::FPS_UPPER_LIMIT {
            self.framecount = 0;
            self.rate = rate;
            self.rateticks = 1000.0 / rate as f32;
            Ok(())
        } else {
            Err(String::from("Rate outside Limit"))
        }
    }

    /// Return the current target framerate in Hz.
    pub fn get_framerate(&self) -> u32 {
        self.rate
    }

    /// Return the current framecount.
    pub fn get_frame_count(&self) -> u32 {
        self.framecount
    }

    /// Delay execution to maintain a constant framerate and calculate fps.
    pub fn delay(&mut self) -> u32 {
        let current_ticks : u32;
	    let target_ticks : u32;
	    let the_delay : u32;
	    let time_passed : u32;
	    /*
	    * Next frame 
	    */
	    self.framecount += 1;
	    /*
	    * Get/calc ticks 
	    */
	    current_ticks = self.get_ticks();
	    time_passed = current_ticks - self.lastticks;
	    self.lastticks = current_ticks;
	    target_ticks = self.baseticks + (self.framecount as f32 * self.rateticks) as u32;
	    if current_ticks <= target_ticks {
		    the_delay = target_ticks - current_ticks;
		    self.wait_ticks(the_delay);
	    } else {
		    self.framecount = 0;
		    self.baseticks = self.get_ticks();
        }
	    return time_passed;
    }
}