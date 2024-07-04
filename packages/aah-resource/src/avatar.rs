use std::{fs, path::Path};

// char_{id}_{name}_{type}

// pub fn get_char_avatars<P: AsRef<Path>>(char_id: u32, res_dir: P) -> Vec<DynamicImage> {
//     let res_dir = res_dir.as_ref();
//     let p = res_dir.join("avatars").join("char").join(format!("char_{:03}", char_id));
//     image::open("")
// }

#[cfg(test)]
mod test {
    use std::{fs, path::Path};

    #[test]
    /// 用于预处理图像，按照角色 id 文件夹存放
    fn foo() {
        let files = fs::read_dir("../../resources/avatars/char").unwrap();
        for file in files {
            let file = file.unwrap();
            let src_path = file.path();
            if !(file.file_type().unwrap().is_file()
                && src_path.extension().and_then(|s| s.to_str()) == Some("png"))
            {
                println!("skipping {src_path:?}...");
                continue;
            }

            let filename = file.file_name();
            let splitted_filename = filename.to_str().unwrap().split("_").collect::<Vec<&str>>();
            println!("{:?}", splitted_filename);
            let new_filename = splitted_filename[2..].join("_");

            let target_dir = src_path.parent().unwrap().join(splitted_filename[1]);
            let target_path = target_dir.join(new_filename);

            println!("moving {filename:?} from {src_path:?} to {target_path:?}...",);
            fs::create_dir_all(&target_dir).unwrap();
            fs::copy(&src_path, &target_path).unwrap();
            fs::remove_file(&src_path).unwrap();
        }
    }
}
