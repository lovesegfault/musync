use std::path;
mod probe;
mod sync;
mod test;


fn soft_quit<E>(error: &E) -> !
    where E: ::std::error::Error
{
    println!("Error: {}", error);
    std::process::exit(1);
}


fn main() {
    //let src = path::Path::new("/mnt/Media/Music/");
    //let dst = path::Path::new("/mnt/veracrypt1/Music/");

    let test_path = path::Path::new("/home/meurer/test");
    test::mk_test_dirs(&test_path, None, 10);

    let src = path::Path::new("/home/meurer/test/a");
    let dst = path::Path::new("/home/meurer/test/b");

    /*
    let mut obj_list = probe::objects(src, None).unwrap_or_else(|e| soft_quit(&e));
    for obj in obj_list {
        println!("{}", obj.display());
    }

    println!("\n\n{} -> {}\n\n", src.display(), dst.display());

    sync::copy(src, dst, None).unwrap_or_else(|e| soft_quit(&e));


    obj_list = probe::objects(dst, None).unwrap_or_else(|e| soft_quit(&e));
    for obj in obj_list {
        println!("{}", obj.display());
    }
    */
    /*let change_list = probe::changed(src, dst, None).unwrap_or_else(|e| soft_quit(&e));
    println!("\n{:?} changed files: ", change_list.len());
    for change in change_list{
        println!("{:?}", change.display());
    }
*/
}
