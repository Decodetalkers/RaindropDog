use crate::tool::{get_v2ray, remove_quotation, write_json};
use gtk::prelude::*;
use serde_json::Value;
use std::{cell::RefCell, env, fs::File, io::prelude::*, path::Path};
thread_local! {
    static GLOBAL: RefCell<Option<gtk::Box>> = RefCell::new(None);
}
pub fn create_sub_window(
    application: &gtk::Application,
    title: &str,
    func: fn(&gtk::TreeStore, Vec<String>, Vec<String>),
    model: &gtk::TreeStore,
    mainwindow: &gtk::ApplicationWindow,
) {
    mainwindow.set_deletable(false);

    let notebook = gtk::Notebook::new();
    let window = gtk::Window::new(gtk::WindowType::Toplevel);

    application.add_window(&window);

    window.set_title(title);
    window.set_default_size(400, 40);
    //window.set_resizable(false);

    // urls设置界面
    {
        let boxs = gtk::Box::new(gtk::Orientation::Vertical, 10);
        let boxs2 = gtk::Box::new(gtk::Orientation::Vertical, 10);
        let urls_input = gtk::Entry::new();
        let button_box = gtk::ButtonBox::new(gtk::Orientation::Horizontal);
        button_box.set_layout(gtk::ButtonBoxStyle::End);
        let button = gtk::Button::with_label("Input");
        let button2 = gtk::Button::with_label("Upload");
        button_box.pack_start(&button, false, false, 0);
        button_box.pack_start(&button2, false, false, 0);
        boxs.pack_start(&boxs2, true, false, 0);
        boxs.pack_start(&urls_input, true, false, 0);
        boxs.pack_start(&button_box, false, false, 0);

        //创建快捷键
        let accel_group = gtk::AccelGroup::new();
        window.add_accel_group(&accel_group);
        //添加用户组
        let (key, modifier) = gtk::accelerator_parse("Return");
        button.add_accelerator("clicked", &accel_group, key, modifier, gtk::AccelFlags::VISIBLE);
        //创建结束
        //预加载
        let home = env::var("HOME").unwrap();
        let location = home + "/.config/gv2ray/urls.json";
        let path = Path::new(location.as_str());
        //let display = path.display();
        let mut file = match File::open(&path) {
            // `io::Error` 的 `description` 方法返回一个描述错误的字符串。
            Err(_) => {
                let path2 = Path::new(location.as_str());
                let display2 = path2.display();
                let mut file2 = match File::create(&path2) {
                    Err(why) => panic!("couldn't create {}: {}", display2, why.to_string()),
                    Ok(file2) => file2,
                };
                let mut storge2: String = String::new();
                storge2.push_str("[]");
                // 将 `LOREM_IPSUM` 字符串写进 `file`，返回 `io::Result<()>`
                if let Err(why) = file2.write_all(storge2.as_bytes()) {
                    panic!("couldn't write to {}: {}", display2, why.to_string())
                }
                let path3 = Path::new(location.as_str());
                File::open(&path3).unwrap()
            }
            Ok(file) => file,
        };
        let mut ss = String::new();
        match file.read_to_string(&mut ss) {
            Err(_) => {}
            Ok(_) => {
                //json 出现问题返回空
                let v: Value = match serde_json::from_str(ss.as_str()) {
                    Err(_) => Value::Null,
                    Ok(ouput) => ouput,
                };
                let mut index = 0;
                while v[index] != Value::Null {
                    create_url(
                        &boxs2,
                        remove_quotation(v[index]["name"].to_string()),
                        remove_quotation(v[index]["url"].to_string()),
                    );
                    index += 1;
                }
            }
        }
        // 加载结束
        GLOBAL.with(move |global| {
            *global.borrow_mut() = Some(boxs2);
        });
        button.connect_clicked(move |_| {
            let urls = urls_input.text().to_string();
            urls_input.set_text("");
            GLOBAL.with(move |global| {
                if let Some(ref bos) = *global.borrow() {
                    create_url(bos, urls, "".to_string());
                }
            });
            //create_url(&boxs2, urls);
        });
        button2.connect_clicked(glib::clone!(@weak model =>move |_|{
            let mut output : Vec<String> = vec![];
            let mut names : Vec<String> = vec![];
            let mut json : String = "[".to_string();
            GLOBAL.with(move |global|{
                if let Some(ref boxs2) = *global.borrow(){
                    for index in boxs2.children() {
                        // 通过gtk的子类的事件，获取到entry的控件，从而获取内容
                        let temp : String = index.downcast_ref::<gtk::Box>().unwrap().children()[0].downcast_ref::<gtk::Label>().unwrap().text().to_string();
                        names.push(temp.clone());
                        let temp2 : String = index.downcast_ref::<gtk::Box>().unwrap().children()[1].downcast_ref::<gtk::Entry>().unwrap().text().to_string();
                        output.push(temp2.clone());
                        json.push_str(
                            format!("
     {{
        \"name\":\"{}\",
        \"url\":\"{}\"
     }},",temp,temp2).as_str());

                    }
                    json.pop();
                    json.push_str("\n]");
                }
                write_json("/.config/gv2ray/urls.json".to_string(), json);
                //println!("{:?}",output);
                func(&model,output,names);
            })
        }));
        create_tab(&notebook, "urls", boxs.upcast());
    }
    //{
    //    let boxs = gtk::Box::new(gtk::Orientation::Vertical, 10);
    //    let urls_input = gtk::Entry::new();
    //    let button_box = gtk::ButtonBox::new(gtk::Orientation::Horizontal);
    //    button_box.set_layout(gtk::ButtonBoxStyle::End);
    //    let button = gtk::Button::with_label("Input");
    //    button_box.pack_start(&button, false, false, 0);
    //    boxs.pack_start(&urls_input, true, false, 0);
    //    boxs.pack_start(&button_box, false, false, 0);
    //    //button.connect_clicked(glib::clone!(@weak model =>move |_|{
    //    //        //model.clear();
    //    //        let input: String = urls_input.text().to_string();
    //    //        let temp : Vec<String> = vec![input];
    //    //        func(&model, temp);
    //    //}));
    //    create_tab(&notebook, "urls", boxs.upcast());
    //}
    {
        let boxs = gtk::Box::new(gtk::Orientation::Vertical, 10);
        let urls_input = gtk::Entry::new();
        let button_box = gtk::ButtonBox::new(gtk::Orientation::Horizontal);
        button_box.set_layout(gtk::ButtonBoxStyle::End);
        let button = gtk::Button::with_label("set");
        button_box.pack_start(&button, false, false, 0);
        boxs.pack_start(&urls_input, true, false, 0);
        boxs.pack_start(&button_box, false, false, 0);
        let (_, v2ray) = get_v2ray();
        urls_input.set_text(&v2ray);
        button.connect_clicked(move |_| {
            //model.clear();
            if urls_input.text() != "" {
                write_json(
                    "/.config/gv2ray/v2core.json".to_string(),
                    format!(
                        "{{
    \"v2core\":\"{}\"
}}",
                        urls_input.text().to_string()
                    ),
                );
            }
        });
        create_tab(&notebook, "v2ray", boxs.upcast());
    }

    window.add(&notebook);
    window.connect_delete_event(
        // drop的信号
        glib::clone!(@weak mainwindow => @default-return Inhibit(false), move |_,_| {
            mainwindow.set_deletable(true);
            Inhibit(false)
        }),
    );
    window.show_all();
    // Once the new window has been created, we put it into our hashmap so we can update its
    // title when needed.
}
fn create_url(boxs: &gtk::Box, names: String, urls: String) {
    let url_box = gtk::Box::new(gtk::Orientation::Horizontal, 10);
    let label = gtk::Label::new(Some(&names));
    let urls_input = gtk::Entry::new();
    urls_input.set_text(&urls);
    let button = gtk::Button::with_label("remove");
    url_box.pack_start(&label, true, false, 0);
    url_box.pack_start(&urls_input, true, true, 0);
    url_box.pack_start(&button, false, false, 0);
    boxs.pack_start(&url_box, false, false, 0);
    button.connect_clicked(glib::clone!(@weak urls_input,@weak boxs => move |_|{
        boxs.remove(&url_box);
    }));
    boxs.show_all();
}
fn create_tab(notebook: &gtk::Notebook, title: &str, widget: gtk::Widget) {
    let label = gtk::Label::new(Some(title));
    notebook.append_page(&widget, Some(&label));
}
