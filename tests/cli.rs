use std::io::Write;
use std::process::{Child, Command, ExitStatus, Stdio};
use std::thread;
use std::time::{Duration, Instant};

fn spawn_emulator() -> Child {
    Command::new(env!("CARGO_BIN_EXE_sabrina_colour"))
        .stdin(Stdio::piped())
        .spawn()
        .expect("start the emulator")
}

fn wait_for_exit(mut child: Child) -> ExitStatus {
    let timeout = Duration::from_secs(3);
    let deadline = Instant::now() + timeout;

    loop {
        if let Some(status) = child.try_wait().expect("check emulator exit status") {
            return status;
        }
        if Instant::now() >= deadline {
            child.kill().expect("stop the timed-out emulator");
            child.wait().expect("reap the timed-out emulator");
            panic!("emulator did not exit within {timeout:?}");
        }
        thread::sleep(Duration::from_millis(10));
    }
}

#[test]
fn quits_on_q() {
    let mut child = spawn_emulator();
    let mut stdin = child.stdin.take().expect("open emulator stdin");
    stdin.write_all(b"\nq\n").expect("send q to the emulator");

    // Keep stdin open so EOF cannot account for a successful exit.
    assert!(wait_for_exit(child).success());
    drop(stdin);
}
