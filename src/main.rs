extern crate ansi_escapes;
extern crate carboxyl;

use carboxyl::*;

fn main() {
    use std::io::{self, Write};

    let sink_tmp = Sink::new();
    let sink_hmd = Sink::new();
    let stream_tmp = sink_tmp.stream();
    let stream_hmd = sink_hmd.stream();

    let signal_tmp = stream_tmp.hold(0.0);
    let signal_hmd = stream_hmd.hold(0.0);
    let signal_di = lift!(
        |tmp, hmd| { 0.81 * tmp + 0.01 * hmd * (0.99 * tmp - 14.3) + 46.3 },
        &signal_tmp,
        &signal_hmd
    );

    let signal_fan = Signal::cyclic(|fan| {
        let signal_th = fan
            .snapshot(&stream_tmp, |fan, _| 75.0 + if fan { -0.5 } else { 0.5 })
            .hold(75.5);
        lift!(|th, di| di >= th, &signal_th, &signal_di)
    });

    let (mut tmp, mut hmd, mut dtmp, mut dhmd) = (30.0, 60.0, 0.5, 1.0);
    loop {
        if tmp > 35.0 || tmp < 20.0 {
            dtmp = -dtmp;
        }
        if hmd > 80.0 || hmd < 50.0 {
            dhmd = -dhmd;
        }
        tmp += dtmp;
        hmd += dhmd;

        sink_tmp.send_async(tmp);
        sink_hmd.send_async(hmd);

        let di = signal_di.sample();
        let fan = signal_fan.sample();
        println!(
            "tmp={:2.2}, hmd={:2.2}, di={:2.2}, fan: {}",
            tmp,
            hmd,
            di,
            if fan { "ON" } else { "OFF" }
        );
        std::thread::sleep(std::time::Duration::from_micros(200000));
        print!("{}", ansi_escapes::EraseLines(2));
        io::stdout().flush().unwrap();
    }
}

// -> tmp ---\
//            v
// -> hmd --> di --> fan
//                  /  ^
//                 >th-/

// module FanController  # module name
// in  tmp : Double,     # temperature sensor
//     hmd : Double      # humidity sensor
// out fan : Bool,       # fan switch
//     di  : Double      # discomfort index
// use Std
//
// # discomfort (temperature-humidity) index
// node di = 0.81 *. tmp +. 0.01 *. hmd *. (0.99 *. tmp -. 14.3) +. 46.3
//
// # fan switch
// node init[False] fan = di >=. th
//
// # hysteresis offset
// node th = 75.0 +. (if fan@last then -.0.5 else 0.5)
