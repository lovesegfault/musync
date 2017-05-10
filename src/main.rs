use std::path;
mod probe;
mod sync;


fn soft_quit<E>(error: E) -> !
    where E: ::std::error::Error
{
    println!("Error: {}", error);
    std::process::exit(1);
}


fn main() {
    let src = path::Path::new("/home/meurer/test/a");
    let dst = path::Path::new("/home/meurer/test/b");
    let mut obj_list = probe::objects(src, None).unwrap_or_else(|e| soft_quit(e));
    for obj in obj_list {
        println!("{}", obj.display());
    }

    sync::copy(src, dst, None).unwrap_or_else(|e| soft_quit(e));

    obj_list = probe::objects(src, None).unwrap_or_else(|e| soft_quit(e));
    for obj in obj_list {
        println!("{}", obj.display());
    }

}
