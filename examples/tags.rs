extern crate dkregistry;
extern crate tokio_core;

use std::{error, boxed};
use tokio_core::reactor::Core;

type Result<T> = std::result::Result<T, boxed::Box<error::Error>>;

fn main() {
    let registry = match std::env::args().nth(1) {
        Some(x) => x,
        None => "registry-1.docker.io".into(),
    };

    let image = match std::env::args().nth(2) {
        Some(x) => x,
        None => "library/debian".into(),
    };
    println!("[{}] requesting tags for image {}", registry, image);

    let user = std::env::var("DKREG_USER").ok();
    if user.is_none() {
        println!("[{}] no $DKREG_USER for login user", registry);
    }
    let password = std::env::var("DKREG_PASSWD").ok();
    if password.is_none() {
        println!("[{}] no $DKREG_PASSWD for login password", registry);
    }

    let res = run(&registry, user, password, &image);

    if let Err(e) = res {
        println!("[{}] {}", registry, e);
        std::process::exit(1);
    };
}

fn run(host: &str, user: Option<String>, passwd: Option<String>, image: &str) -> Result<()> {
    let mut tcore = try!(Core::new());
    let mut dclient = try!(dkregistry::v2::Client::configure(&tcore.handle())
        .registry(host)
        .insecure_registry(false)
        .username(user)
        .password(passwd)
        .build());

    let futcheck = try!(dclient.is_v2_supported());
    let supported = try!(tcore.run(futcheck));
    if !supported {
        return Err("API v2 not supported".into());
    }

    try!(dclient.login(vec![&format!("repository:{}:pull", image)]));

    let futauth = try!(dclient.is_auth());
    if !try!(tcore.run(futauth)) {
        return Err("login failed".into());
    }

    let fut_tags = try!(dclient.get_tags(image, None));
    let tags = try!(tcore.run(fut_tags));

    println!("{:?}", tags);

    return Ok(());
}
