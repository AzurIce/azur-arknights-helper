//! some utils

use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt, EnvFilter};

pub fn init_logger() {
    // let indicatif_layer = IndicatifLayer::new();

    let filter_layer = EnvFilter::try_from_default_env()
        .or_else(|_| EnvFilter::try_new("info"))
        .unwrap();

    tracing_subscriber::registry()
        .with(filter_layer)
        .with(
            tracing_subscriber::fmt::layer(), // .with_level(false)
                                              // .with_target(false)
                                              // .without_time()
                                              // .with_writer(indicatif_layer.get_stderr_writer()),
        )
        // .with(indicatif_layer)
        .init();
}

pub mod resource {
    //! resource related utils
    use std::{fs, path::Path};

    use image::DynamicImage;

    /// 从 `{res_path}/resources/templates/1920x1080` 目录中根据文件名称获取模板
    pub fn get_template<S: AsRef<str>, P: AsRef<Path>>(
        template_filename: S,
        res_dir: P,
    ) -> Result<DynamicImage, String> {
        let filename = template_filename.as_ref();
        let res_dir = res_dir.as_ref();

        let path = res_dir.join("templates").join("1920x1080").join(filename);
        let image = image::open(path).map_err(|err| format!("template not found: {err}"))?;
        Ok(image)
    }

    /// 从 `{res_path}/resources/avatars` 下加载全部干员内部名称
    pub fn get_opers<P: AsRef<Path>>(res_dir: P) -> Vec<String> {
        let res_dir = res_dir.as_ref();
        let mut opers: Vec<(u32, String)> = vec![];
        let char_avatars = fs::read_dir(res_dir.join("avatars").join("char")).unwrap();
        for char_avatar in char_avatars {
            let char_avatar = char_avatar.unwrap();
            if char_avatar.file_type().unwrap().is_file() {
                continue;
            }
            let mut name = fs::read_dir(char_avatar.path()).unwrap();
            let f = name.next().unwrap().unwrap();
            let name = f.file_name().into_string().unwrap();
            let name = name.split('_').collect::<Vec<&str>>()[0];
            let name = name.split('.').collect::<Vec<&str>>()[0];
            let num = char_avatar.file_name().into_string().unwrap();
            opers.push((num.parse().unwrap(), name.to_string()));
        }
        opers
            .into_iter()
            .map(|(num, name)| format!("char_{num:03}_{name}.png"))
            .collect()
    }

    /// 输入干员列表和资源路径，以 `Vec<(名,头像)>` 形式返回这些干员的所有头像列表
    pub fn get_opers_avatars<S: AsRef<str>, P: AsRef<Path>>(
        opers: Vec<S>,
        res_dir: P,
    ) -> Result<Vec<(String, DynamicImage)>, String> {
        let res_dir = res_dir.as_ref();
        let oper_images = opers
            .iter()
            .map(|s| {
                let s = s.as_ref();
                get_oper_avatars(s, res_dir).map(|imgs| {
                    imgs.into_iter()
                        .map(|img| (s.to_string(), img))
                        .collect::<Vec<(String, DynamicImage)>>()
                })
            })
            .collect::<Result<Vec<Vec<(String, DynamicImage)>>, String>>()?;
        let oper_images = oper_images
            .into_iter()
            .flatten()
            .collect::<Vec<(String, DynamicImage)>>();
        Ok(oper_images)
    }

    /// 从 `<res_dir>/avatars/` 获取指定角色的所有头像图片
    pub fn get_oper_avatars<S: AsRef<str>, P: AsRef<Path>>(
        oper: S,
        res_dir: P,
    ) -> Result<Vec<DynamicImage>, String> {
        let oper = oper.as_ref();
        let res_dir = res_dir.as_ref();

        // println!("{:?}", oper);
        let components = oper.split("_").collect::<Vec<&str>>();
        let path = res_dir.join("avatars").join("char").join(components[1]);
        // println!("{:?}", path);

        let mut images = vec![];
        for f in fs::read_dir(path).unwrap() {
            let f = f.unwrap();
            println!("{:?}", f.path());
            let image = image::open(f.path()).map_err(|err| format!("avatar not found: {err}"))?;
            // let image = image.crop_imm(30, 20, 100, 120);
            images.push(image);
        }
        Ok(images)
    }
}
