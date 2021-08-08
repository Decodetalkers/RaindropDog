mod multi;
mod spider;
mod tool;
#[macro_use]
extern crate lazy_static;
use futures::executor::block_on;
use gtk::prelude::*;
use gtk::{
    ApplicationWindow, CellRendererText, Label, Orientation, TreeStore, TreeView, TreeViewColumn,
    WindowPosition,
};
//use serde_json::Result;
use serde_json::Value;
use spider::{ascii_to_char, get_the_key};
use std::{
    cell::RefCell,
    env,
    fs::{self, File},
    io::prelude::*,
    path::Path,
    process::Command,
    sync::{
        //mpsc,
        Arc,
        RwLock,
    },
    thread,
    time::Duration,
};
use tool::Urls;
//use std::cell::RefCell;
#[derive(Copy, Clone)]
struct Active {
    is_running: (i32, i32),
    local: (i32, i32),
}

struct Ui {
    running_button: gtk::Button,
    reload_button: gtk::Button,
    ui_label: gtk::Label,
    func_label: gtk::Label,
    add_label: gtk::Label,
    port_label: gtk::Label,
    url_label: gtk::Label,
}
//记录根节点的名字以及内容，为接下来存储多个信息做铺垫
struct AllUrls {
    name: String,
    content: Vec<Urls>,
}
//这边是需要更改的全局变量，设置到thread_local里，理论上应该设置成全局变量？但是gtk给的就是thread_local..
thread_local! {
    static GLOBALURL: RefCell<Option<Vec<AllUrls>>> = RefCell::new(None);
    static GLOBAL: RefCell<Option<Ui>> = RefCell::new(None);
    static GLOBALTEXT: RefCell<Option<gtk::TextView>> = RefCell::new(None);
    static GLOBAL2: RefCell<Active> = RefCell::new(Active {
        is_running: (0, -1),
        local: (0, 0),
    });
    //static GLOBAL3: RefCell<bool> = RefCell::new(false);
//    static GLOBALTHREAD: RefCell<std::process::Child>=RefCell::new(
//        Command::new("ls")
//        .spawn()
//        .expect("error"));
    //static GLOBALCOMMUNITY: RefCell<(mpsc::Sender<bool>,mpsc::Receiver<bool>)> = RefCell::new(mpsc::channel());
}
//全局变量
lazy_static! {
    static ref GLOBALARC: Arc<RwLock<bool>> = Arc::new(RwLock::new(false));
}
//杀死子进程
fn kill() {
    println!("stop");
    let mut num = GLOBALARC.write().unwrap();
    *num = true;
}
fn run(name: &Urls, text: &gtk::TextView) {
    //不管如何事先先状态清除
    let mut num2 = GLOBALARC.write().unwrap();
    *num2 = false;
    let mut json = String::new();
    let output = name.get_the_json();
    json.push_str(output.as_str());
    let home = env::var("HOME").unwrap();
    let location = home + "/.config/gv2ray/running.json";
    let path2 = Path::new(location.as_str());
    //let display = path.display();
    //let path2 = Path::new("storage.json");
    let display2 = path2.display();
    let mut file2 = match File::create(&path2) {
        Err(why) => panic!("couldn't create {}: {}", display2, why.to_string()),
        Ok(file2) => file2,
    };
    if let Err(why) = file2.write_all(json.as_bytes()) {
        panic!("couldn't write to {}: {}", display2, why.to_string())
    }

    let (home2, content) = tool::get_v2ray();
    let mut running = Command::new(content)
        .arg("-config")
        .arg(home2 + "/.config/gv2ray/running.json")
        .stdout(std::process::Stdio::piped())
        .spawn()
        .expect("failed");
    let mut stream = running.stdout.take().expect("!stdout");
    //let mut output = running.stdout;
    //let s = String::from_utf8_lossy(&output.unwrap());
    let text_buffer = text.buffer().expect("none");
    let (tx, rx) = glib::MainContext::channel(glib::PRIORITY_DEFAULT);
    // 将stdout的内容通过thread获取，通过channel传到textview里
    thread::Builder::new()
        .name("child_stream_to_vec".into())
        .spawn(move || {
            let mut storage: String = String::new();
            loop {
                let mut buf = [0];
                match stream.read(&mut buf) {
                    Err(err) => {
                        println!("{}] Error reading from stream: {}", line!(), err);
                        break;
                    }
                    Ok(got) => {
                        if got == 0 {
                            print!("The pipe break");
                            break;
                        } else if got == 1 {
                            let index = ascii_to_char(buf[0]);
                            //println!("{}", buf[0]);
                            storage.push(index);
                            //println!("storage is {}", storage);
                            //println!("{}",buf[0]);
                            tx.send(storage.clone()).expect("error");
                        } else {
                            println!("{}] Unexpected number of bytes: {}", line!(), got);
                            break;
                        }
                    }
                }
            }
            //如果超时就杀死这个变量
            drop(stream);
        })
        .expect("!thread");
    rx.attach(None, move |text| {
        //println!("send is {}",text);
        text_buffer.set_text(&text);
        glib::Continue(true)
    });
    thread::spawn(move || {
        loop {
            fn test() -> bool {
                let nums = Arc::clone(&GLOBALARC);
                let num = nums.read().unwrap();
                thread::sleep(Duration::from_millis(10));
                *num
            }
            if test() {
                let mut num2 = GLOBALARC.write().unwrap();
                *num2 = false;

                //println!("is break");
                break;
            }
        }
        //println!("break");
        running.kill().expect("error");
        //wait是必要的，回收进程
        running.wait().unwrap();
        drop(running);
    });
}

fn create_storage_before() {
    let home = env::var("HOME").unwrap();
    fs::create_dir_all(home + "/.config/gv2ray").unwrap();
}
fn create_and_fill_model_before(model: &TreeStore) {
    model.clear();
    create_storage_before();
    let home = env::var("HOME").unwrap();
    let location = home + "/.config/gv2ray/storage.json";
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
            let mut all_urls: Vec<AllUrls> = vec![];
            while v[index] != Value::Null {
                let mut urls: Vec<Urls> = vec![];
                let mut index2 = 0;
                //获取名字
                let name = tool::remove_quotation(v[index]["name"].to_string());
                let iter = model.insert_with_values(None, None, &[(0, &name)]);
                while v[index]["urls"][index2] != Value::Null {
                    let the_url = v[index]["urls"][index2]["url"].to_string();
                    let length = the_url.len();
                    let instore = &the_url[1..length - 1];
                    let url = Urls {
                        urls: instore.to_string(),
                        func: v[index]["urls"][index2]["func"].to_string(),
                        add: v[index]["urls"][index2]["add"].to_string(),
                        aid: v[index]["urls"][index2]["aid"].to_string(),
                        host: v[index]["urls"][index2]["host"].to_string(),
                        id: v[index]["urls"][index2]["id"].to_string(),
                        net: v[index]["urls"][index2]["net"].to_string(),
                        path: v[index]["urls"][index2]["path"].to_string(),
                        port: v[index]["urls"][index2]["port"].to_string(),
                        ps: v[index]["urls"][index2]["ps"].to_string(),
                        tls: v[index]["urls"][index2]["tls"].to_string(),
                        typpe: v[index]["urls"][index2]["type"].to_string(),
                    };
                    model.insert_with_values(
                        Some(&iter),
                        None,
                        &[(
                            0,
                            &(tool::remove_quotation(v[index]["urls"][index2]["ps"].to_string())),
                        )],
                    );
                    urls.push(url);
                    index2 += 1;
                }
                index += 1;
                all_urls.push(AllUrls {
                    name,
                    content: urls,
                })
            }
            GLOBALURL.with(move |global| {
                *global.borrow_mut() = Some(all_urls);
            });
        }
    }
}

// 生成tree
fn create_and_fill_model(model: &TreeStore, temp: Vec<String>, name: Vec<String>) {
    create_storage_before();
    model.clear();
    let future = get_the_key(temp);
    let output: Vec<Vec<String>> = block_on(future).unwrap();
    let mut input: Vec<Vec<String>> = vec![];
    let mut all_urls: Vec<AllUrls> = vec![];
    let mut storge: String = String::new();
    storge.push('[');
    storge.push('\n');

    for (pair, aname) in output.into_iter().zip(name.clone().into_iter()) {
        let mut urls: Vec<Urls> = vec![];
        storge.push_str(
            format!(
                "{{
    \"name\":\"{}\",
    \"urls\":[",
                aname
            )
            .as_str(),
        );
        let mut input_in: Vec<String> = vec![];
        for pair2 in pair.into_iter() {
            let url_local = Urls::new(pair2);
            let temp = url_local.ps.clone();
            urls.push(url_local.clone());
            //let temp = pair2.clone();
            input_in.push(temp);
            storge.push_str(
                format!(
                    "   {{
        \"func\":{},
        \"url\":\"{}\",
        \"add\":{},
        \"aid\":{},
        \"host\":{},
        \"id\":{},
        \"net\":{},
        \"path\":{},
        \"port\":{},
        \"ps\":{},
        \"tls\":{},
        \"type\":{}
    }},\n",
                    url_local.func,
                    url_local.urls,
                    url_local.add,
                    url_local.aid,
                    url_local.host,
                    url_local.id,
                    url_local.net,
                    url_local.path,
                    url_local.port,
                    url_local.ps,
                    url_local.tls,
                    url_local.typpe
                )
                .as_str(),
            );
        }
        all_urls.push(AllUrls {
            name: aname,
            content: urls,
        });
        storge.pop();
        storge.pop();
        storge.push('\n');
        storge.push_str("   ]\n},\n");
        input.push(input_in);
    }
    storge.pop();
    storge.pop();
    storge.push('\n');
    storge.push(']');
    //防止获取信息后覆盖，但是结果为空
    if storge.len() > 10 {
        let home = env::var("HOME").unwrap();
        let location_back = home.clone() + "/.config/gv2ray/storage_back.json";
        let back_path = Path::new(location_back.as_str());
        if back_path.exists() && fs::remove_file(back_path).is_ok() {}
        let location = home.clone() + "/.config/gv2ray/storage.json";
        let location2 = location.clone();
        let path2 = Path::new(location2.as_str());
        if path2.exists() {
            //if let Ok(_)=fs::copy(location, home+"/.config/gv2ray/storage_back.json"){};
            if fs::copy(location, home + "/.config/gv2ray/storage_back.json").is_ok() {};
            GLOBAL.with(move |global| {
                if let Some(ref ui) = *global.borrow() {
                    ui.reload_button.show();
                    ui.reload_button.set_label("reload");
                }
            });
        }
        //let display = path.display();
        //let path2 = Path::new("storage.json");
        let display2 = path2.display();
        let mut file2 = match File::create(&path2) {
            Err(why) => panic!("couldn't create {}: {}", display2, why.to_string()),
            Ok(file2) => file2,
        };

        // 将 `LOREM_IPSUM` 字符串写进 `file`，返回 `io::Result<()>`
        if let Err(why) = file2.write_all(storge.as_bytes()) {
            panic!("couldn't write to {}: {}", display2, why.to_string())
        }
        GLOBALURL.with(move |global| {
            *global.borrow_mut() = Some(all_urls);
        });
        let entries = &input;
        for (entry, aname) in entries.iter().zip(name.iter()) {
            let iter = model.insert_with_values(None, None, &[(0, &aname)]);
            for entry2 in entry.iter() {
                model.insert_with_values(
                    Some(&iter),
                    None,
                    &[(0, &(tool::remove_quotation(entry2.to_string())))],
                );
            }
        }
    };
    //model
}

fn append_column(tree: &TreeView, id: i32) {
    let column = TreeViewColumn::new();
    let cell = CellRendererText::new();

    column.pack_start(&cell, true);
    // Association of the view's column with the model's `id` column.
    column.add_attribute(&cell, "text", id);
    tree.append_column(&column);
}

fn create_and_setup_view() -> TreeView {
    // Creating the tree view.
    let tree = TreeView::new();

    tree.set_headers_visible(false);
    // Creating the two columns inside the view.
    // 去除第二个数字的显示
    append_column(&tree, 0);
    //append_column(&tree, 1);
    tree
}

fn build_ui(application: &gtk::Application) {
    let window = ApplicationWindow::new(application);

    window.set_title("RainDropDog");
    window.set_position(WindowPosition::Center);
    window.set_size_request(300, 300);

    // Creating a vertical layout to place both tree view and label in the window.
    //
    // 使用paned而不是box，为了分屏和调整大小
    let vertical_layout = gtk::Paned::new(Orientation::Horizontal);

    // Creation of the label.
    let label = Label::new(None);
    let label_func = Label::new(Some("func:"));
    label_func.set_justify(gtk::Justification::Left);
    let label_add = Label::new(Some("add:"));
    label_add.set_justify(gtk::Justification::Left);
    let label_port = Label::new(Some("port:"));
    label_port.set_justify(gtk::Justification::Left);
    let label_url = Label::new(Some("url"));
    label_url.set_justify(gtk::Justification::Left);
    label_url.set_max_width_chars(10);
    let text_view = gtk::TextView::new();
    text_view.set_editable(false);
    let scroll_text = gtk::ScrolledWindow::new(gtk::NONE_ADJUSTMENT, gtk::NONE_ADJUSTMENT);
    scroll_text.set_policy(gtk::PolicyType::Automatic, gtk::PolicyType::Automatic);
    scroll_text.add(&text_view);
    label_url.set_line_wrap(true);
    //label_url.set_wrap(true);
    let v_box = gtk::Box::new(gtk::Orientation::Vertical, 10);
    let h_box = gtk::Box::new(gtk::Orientation::Vertical, 10);
    h_box.pack_start(&label_func, false, true, 0);
    //h_box.pack_start(child, expand, fill, padding)
    h_box.pack_start(&label_add, false, true, 0);
    h_box.pack_start(&label_port, false, true, 0);
    h_box.pack_start(&label_url, false, true, 0);
    h_box.pack_start(&label, false, true, 0);
    h_box.pack_start(&scroll_text, true, true, 0);
    let button_box = gtk::ButtonBox::new(gtk::Orientation::Horizontal);
    button_box.set_layout(gtk::ButtonBoxStyle::End);
    let button1 = gtk::Button::with_label("new");
    let button2 = gtk::Button::with_label("edit");
    let button3 = gtk::Button::with_label("reload");
    {
        let home = env::var("HOME").unwrap();
        let location = home + "/.config/gv2ray/storage_back.json";
        let path2 = Path::new(location.as_str());
        if !path2.exists() {
            button3.set_label("None");
            button3.set_visible(false);
        }
    }

    button_box.pack_start(&button1, false, false, 0);
    button_box.pack_start(&button2, false, false, 0);
    button_box.pack_start(&button3, false, false, 0);

    v_box.pack_start(&button_box, false, true, 0);
    v_box.pack_start(&h_box, true, true, 0);

    let tree = create_and_setup_view();
    //let temp: Vec<String> = vec![];
    let model = TreeStore::new(&[glib::Type::STRING]);
    create_and_fill_model_before(&model);
    button2.connect_clicked(
        glib::clone!(@weak model,@weak application,@weak window => move |_|{
            multi::create_sub_window(&application, "input urls",create_and_fill_model,&model,&window);
        }),
    );
    //增加回退功能，如果节点不小心炸了，那么历史记录可以退回来
    button3.connect_clicked(glib::clone!(@weak model => move |button|{
        //button.set_visible(false);
        //button.set_layout(Some())
        let home = env::var("HOME").unwrap();
        let location = home.clone() + "/.config/gv2ray/storage.json";
        //let location2 = location.clone();
        let path = Path::new(location.as_str());
        if path.exists() && fs::remove_file(path).is_ok() {}

        let location_back = home + "/.config/gv2ray/storage_back.json";
        let back_path = Path::new(location_back.as_str());
        //if back_path.exists(){
        //    if fs::copy(location_back, location).is_ok(){};
        //}
        if ! (back_path.exists() && fs::copy(location_back, location).is_ok()) {
            button.set_label("None");
        }
        create_and_fill_model_before(&model);


    }));

    //let model = create_and_fill_model(temp);
    // Setting the model into the view.
    tree.set_model(Some(&model));
    let scroll = gtk::ScrolledWindow::new(gtk::NONE_ADJUSTMENT, gtk::NONE_ADJUSTMENT);
    scroll.add(&tree);
    //设置最小的大小
    scroll.set_width_request(400);
    //scroll.set_resize_mode(gtk::ResizeMode::Queue);
    //禁止水平变化
    //左边加scroll，右边加v_box
    vertical_layout.pack1(&scroll, false, true);
    vertical_layout.pack2(&v_box, true, true);
    // Iter 可以获取内容，但是active可以获取目录位置
    // 准确来说，active需要点两次
    // 移动到global里去，主要是为了方便改写和访问，这样变量就不会首主进程影响
    GLOBALTEXT.with(move |global| {
        *global.borrow_mut() = Some(text_view);
    });
    GLOBAL.with(move |global| {
        *global.borrow_mut() = Some(Ui {
            running_button: button1,
            reload_button: button3,
            ui_label: label,
            func_label: label_func,
            add_label: label_add,
            port_label: label_port,
            url_label: label_url,
        });
        if let Some(ref ui) = *global.borrow() {
            ui.running_button.connect_clicked(move |s| {
                GLOBAL2.with(move |global2| {
                    let locall = *global2.borrow();
                    let temp = locall.local;
                    if locall.is_running == locall.local {
                        s.set_label("start");
                        *global2.borrow_mut() = Active {
                            is_running: (0, -1),
                            local: temp,
                        };
                        kill();
                    } else {
                        s.set_label("stop");
                        //println!("{},{}",locall.is_running,locall.local);
                        *global2.borrow_mut() = Active {
                            is_running: temp,
                            local: temp,
                        };
                        GLOBALURL.with(|globalurl| {
                            if let Some(ref url) = *globalurl.borrow() {
                                GLOBALTEXT.with(|globaltext| {
                                    if let Some(ref text) = *globaltext.borrow() {
                                        run(&url[temp.0 as usize].content[temp.1 as usize], text);
                                    }
                                });
                                //run(&url[temp.0 as usize].content[temp.1 as usize],&text_view);
                            }
                        });
                    }
                });
            });
        }
    });
    //active时候更改label

    tree.connect_row_activated(move |_, path, _column| {
        GLOBAL.with(move |global| {
            if let Some(ref ui) = *global.borrow() {
                if path.depth() > 1 {
                    ui.ui_label.set_text(&format!("index{}", path.indices()[1]));
                    GLOBAL2.with(move |global2| {
                        //如果正在运行就杀死
                        if (*global2.borrow()).is_running != (0, -1) {
                            kill();
                        }
                        //let locall = *global2.borrow();
                        *global2.borrow_mut() = Active {
                            is_running: (path.indices()[0], path.indices()[1]),
                            local: (path.indices()[0], path.indices()[1]),
                        };
                        ui.running_button.set_label("stop");
                    });
                    GLOBALURL.with(|globalurl| {
                        if let Some(ref url) = *globalurl.borrow() {
                            GLOBALTEXT.with(|globaltext| {
                                if let Some(ref text) = *globaltext.borrow() {
                                    //run(&url[temp.0 as usize].content[temp.1 as usize],&text);
                                    run(
                                        &url[path.indices()[0] as usize].content
                                            [path.indices()[1] as usize],
                                        text,
                                    );
                                    //[path.indices()[1] as usize]);
                                }
                            });
                            //run(&url[path.indices()[0] as usize].content
                            //[path.indices()[1] as usize]);
                        }
                    });
                }
            }
        });
    });
    // cursor 聚焦时候就更改
    tree.connect_cursor_changed(move |tree_view| {
        GLOBAL.with(move |global| {
            if let Some(ref ui) = *global.borrow() {
                let selection = tree_view.selection();
                if let Some((model, iter)) = selection.selected() {
                    // Now getting back the values from the row corresponding to the
                    // iterator `iter`.
                    //
                    // The `get_value` method do the conversion between the gtk type and Rust.
                    let path = model.path(&iter).expect("no");
                    if path.depth() > 1 {
                        ui.ui_label.set_text(&format!(
                            "Hello {} ",
                            model
                                .value(&iter, 0)
                                .get::<String>()
                                .expect("Treeview selection, column 1"),
                        ));
                        let local2 = (path.indices()[0], path.indices()[1]);
                        GLOBALURL.with(move |global| {
                            if let Some(ref url) = *global.borrow() {
                                ui.func_label.set_text(&format!(
                                    "func: {}",
                                    tool::remove_quotation(
                                        url[local2.0 as usize].content[local2.1 as usize]
                                            .func
                                            .clone()
                                    )
                                    .as_str()
                                ));
                                ui.add_label.set_text(&format!(
                                    "add: {}",
                                    tool::remove_quotation(
                                        url[local2.0 as usize].content[local2.1 as usize]
                                            .add
                                            .clone()
                                    )
                                    .as_str()
                                ));
                                ui.port_label.set_text(&format!(
                                    "port: {}",
                                    tool::remove_quotation(
                                        url[local2.0 as usize].content[local2.1 as usize]
                                            .port
                                            .clone()
                                    )
                                    .as_str()
                                ));
                                ui.url_label.set_text(&format!(
                                    "url: {}",
                                    url[local2.0 as usize].content[local2.1 as usize]
                                        .get_the_link()
                                        .as_str()
                                ));
                                ui.url_label.set_max_width_chars(10);
                                ui.url_label.set_line_wrap(true);
                            }
                        });
                        GLOBAL2.with(move |global2| {
                            let locall = *global2.borrow();
                            let running = locall.is_running;
                            *global2.borrow_mut() = Active {
                                is_running: running,
                                local: local2,
                            };
                            //locall.local = local;
                            if local2 != running {
                                //locall.is_running = path.indices()[0];
                                ui.running_button.set_label("start");
                            } else {
                                ui.running_button.set_label("stop");
                            }
                        });
                    } else {
                        GLOBALURL.with(move |globalurl| {
                            if let Some(ref url) = *globalurl.borrow() {
                                ui.ui_label.set_label(&format!(
                                    "the name is{}",
                                    url[path.indices()[0] as usize].name
                                ));
                            }
                        })
                    }
                }
            }
        });
    });
    // Adding the layout to the window.
    window.connect_delete_event(move |_, _| {
        kill();
        Inhibit(false)
    });
    window.add(&vertical_layout);

    window.show_all();
}

fn main() {
    let application = gtk::Application::new(
        Some("com.github.gtk-rs.examples.simple_treeview"),
        Default::default(),
    );

    application.connect_activate(build_ui);

    application.run();
}
