use std::path;
mod probe;


fn soft_quit(error: String) -> ! {
    println!("Error: {}", error);
    std::process::exit(1);
}


fn main() {
    let src = path::Path::new("/home/meurer/test/a");
    //let dst = path::Path::new("/home/meurer/test/b");
    let file_list = match probe::files(src, -1) {
        Ok(f) => f,
        Err(e) => soft_quit(e.to_string()),
    };
    let dir_list = match probe::directories(src, -1) {
        Ok(f) => f,
        Err(e) => soft_quit(e.to_string()),
    };
    let obj_list = match probe::objects(src, -1){
        Ok(f) => f,
        Err(e) => soft_quit(e.to_string()),
    };
    for file in file_list {
        println!("{}", file.display());
    }
    for dir in dir_list {
        println!("{}", dir.display());
    }
    for obj in obj_list{
        println!("{}", obj.display());
    }
}
