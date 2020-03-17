Add to your cargo.toml:

~~~
[features]
default = ["profile"]
profile = ["screeps-timing"]

[dependencies]
screeps-timing = { git = "https://github.com/Azaril/screeps-timing", optional = true }
serde = "1.0"
serde_json = "1.0"`
~~~

Minimum setup for timing a main loop tick and dumping it to console.

~~~
fn main_loop() {
    #[cfg(feature = "profile")]
    {
        screeps_timing::start_trace(|| screeps::game::cpu::get_used());
    }
    
    game_loop::tick();

    #[cfg(feature = "profile")]
    {
        let trace = screeps_timing::stop_trace();

        if let Some(trace_output) = serde_json::to_string(&trace).ok() {
            info!("{}", trace_output);
        }
    }   
}
~~~
