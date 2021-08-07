use crate::tool::{get_v2ray, write_json};
use gtk::prelude::*;
use std::cell::RefCell;
thread_local! {
    static GLOBAL: RefCell<Option<gtk::Box>> = RefCell::new(None);
}
pub fn create_sub_window(
    application: &gtk::Application,
    title: &str,
    func: fn(&gtk::TreeStore, Vec<String>),
    model: &gtk::TreeStore,
    mainwindow: &gtk::ApplicationWindow,
) {
    mainwindow.set_deletable(false);

    let notebook = gtk::Notebook::new();
    let window = gtk::Window::new(gtk::WindowType::Toplevel);

    application.add_window(&window);

    window.set_title(title);
    window.set_default_size(400, 40);
    window.set_resizable(false);

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
        GLOBAL.with(move |global|{
            *global.borrow_mut() =Some(boxs2);
        });
        button.connect_clicked(move |_|{
            let urls = urls_input.text().to_string();
            GLOBAL.with(move |global|{
                if let Some(ref bos) = *global.borrow(){
                    create_url(bos, urls);
                }
            });
            //create_url(&boxs2, urls);
        });
        button2.connect_clicked(glib::clone!(@weak model =>move |_|{
            let mut output : Vec<String> = vec![];
            GLOBAL.with(move |global|{
                if let Some(ref boxs2) = *global.borrow(){
                    for index in boxs2.children() {
                        // 通过gtk的子类的事件，获取到entry的控件，从而获取内容
                        let temp : String = index.downcast_ref::<gtk::Box>().unwrap().children()[0].downcast_ref::<gtk::Entry>().unwrap().text().to_string();
                        output.push(temp);
                    }
                }
                //println!("{:?}",output);
                func(&model,output);
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
        button.connect_clicked(move |_|{
                //model.clear();
            if urls_input.text() !=""{
            write_json("/.config/gv2ray/v2core.json".to_string(),
                format!("{{
    \"v2core\":\"{}\"
}}",urls_input.text().to_string()));
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
#[allow(dead_code)]
fn create_url(boxs: &gtk::Box,urls: String) {
    let url_box = gtk::Box::new(gtk::Orientation::Horizontal,10);
    let urls_input = gtk::Entry::new();
    urls_input.set_text(&urls);
    let button = gtk::Button::with_label("remove");
    url_box.pack_start(&urls_input, true,false, 0);
    url_box.pack_start(&button, false,false, 0);
    boxs.pack_start(&url_box, false, false, 0);
    button.connect_clicked(glib::clone!(@weak urls_input,@weak boxs => move |_|{
        boxs.remove(&url_box);
    }));
    //println!("{:?}",boxs.children());
    //if boxs.children().len() > 1{
    //    //println!("{:?}",boxs.children()[0]);
    //    //这个可以通过gtk自带的反射获取内容
    //    println!("{}",(boxs.children()[0].downcast_ref::<gtk::Box>().unwrap().children()[0]).downcast_ref::<gtk::Entry>().unwrap().text());
    //}
    //显示所有
    boxs.show_all();
}
fn create_tab(notebook: &gtk::Notebook, title: &str, widget: gtk::Widget) {
    let label = gtk::Label::new(Some(title));
    notebook.append_page(&widget, Some(&label));
}
