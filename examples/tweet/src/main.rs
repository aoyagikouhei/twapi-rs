use twapi::Twapi;

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let user_auth = twapi::UserAuth::new(
        &args[1],
        &args[2],
        &args[3],
        &args[4],
    );
    let res = user_auth.post_statuses_update(&vec![("status", "テ ス ト 4")]);
    println!("{:?}", res);
}
