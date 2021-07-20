use std::cell::RefCell;

use gtk::prelude::*;
use gtk::{
    ApplicationWindow, CellRendererText, Label, ListStore, Orientation, TreeView, TreeViewColumn,
    WindowPosition,
};
#[derive(Copy,Clone)]
struct Active {
    is_running: bool,
}

struct Ui{
    running_button: gtk::Button, 
    ui_label:gtk::Label,
}
thread_local!(
    static GLOBAL: RefCell<Option<Ui>> = RefCell::new(None)
);
thread_local!(
    static GLOBAL2: RefCell<Option<Active>> = RefCell::new(None)
);


fn create_and_fill_model() -> ListStore {
    // Creation of a model with two rows.
    let model = ListStore::new(&[u32::static_type(),String::static_type()]);
    // Filling up the tree view.
    let entries = &["Michel", "Sara", "Liam", "Zelda", "Neo", "Octopus master","Michel", "Sara", "Liam", "Zelda", "Neo", "Octopus master"];
    for (i, entry) in entries.iter().enumerate() {
        model.insert_with_values(None, &[(0,&(i as u32 + 1)),(1, &entry)]);
    }
    model
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
    append_column(&tree, 0);
    append_column(&tree, 1);
    tree
}

fn build_ui(application: &gtk::Application) {
    let window = ApplicationWindow::new(application);

    window.set_title("Simple TreeView example");
    window.set_position(WindowPosition::Center);
    window.set_size_request(300, 300);

    // Creating a vertical layout to place both tree view and label in the window.
    let vertical_layout = gtk::Box::new(Orientation::Horizontal, 0);

    // Creation of the label.
    let label = Label::new(None);
    let v_box = gtk::Box::new(gtk::Orientation::Vertical, 10);
    let button_box = gtk::ButtonBox::new(gtk::Orientation::Horizontal);
    button_box.set_layout(gtk::ButtonBoxStyle::End);
    let button1 = gtk::Button::with_label("new");
    let active :bool = false;
    GLOBAL2.with(move |global|{
        *global.borrow_mut() = Some(Active{
            is_running:active
        });
    });
    let button2 = gtk::Button::with_label("copy");
    button_box.pack_start(&button1, false, false, 0);
    button_box.pack_start(&button2, false, false, 0);

    v_box.pack_start(&button_box, false, true, 0);
    v_box.pack_start(&label, true, true, 0);


    let tree = create_and_setup_view();

    let model = create_and_fill_model();
    // Setting the model into the view.
    tree.set_model(Some(&model));
    let scroll = gtk::ScrolledWindow::new(
        gtk::NONE_ADJUSTMENT,
        gtk::NONE_ADJUSTMENT);
        //Some(&gtk::Adjustment::new(1000000.0, 800000.0,600.0,70000000000.0,60000000.0,6000.0)));
    scroll.add(&tree);
    //设置最小的大小
    scroll.set_width_request(160);
    scroll.set_resize_mode(gtk::ResizeMode::Queue);
    //禁止水平变化
    vertical_layout.pack_start(&scroll, false, true, 0);
    vertical_layout.pack_start(&v_box, true, true, 0);
    // Adding the view to the layout.
    //vertical_layout.add(&scroll);
    // Same goes for the label.
    //vertical_layout.add(&label);

    // The closure responds to selection changes by connection to "::cursor-changed" signal,
    // that gets emitted when the cursor moves (focus changes).
    // Iter 可以获取内容，但是active可以获取目录位置
    // 准确来说，active需要点两次
    GLOBAL.with(move |global|{
        *global.borrow_mut() = Some(Ui{
            running_button:button1,
            ui_label:label,
        });
        if let Some(ref ui) = *global.borrow(){
            ui.running_button.connect_clicked(move |s|{
                GLOBAL2.with(move |global2|{
                    let mut test : bool = true;
                    if let Some(ref active) = *global2.borrow_mut(){
                        test = !active.is_running.clone();
                        if active.is_running{
                            s.set_label("stop");
                        } else {
                            s.set_label("start");
                        }
                    }
                    *global2.borrow_mut() = Some(Active{
                        is_running: test
                    });
                });
            });
        }
    });
    tree.connect_row_activated(move |_, path, _column| {
        GLOBAL.with(move |global|{
            if let Some(ref ui) = *global.borrow(){
                ui.ui_label.set_text(&format!("index{}",path.indices()[0]))
            }
        });
        //println!("{}",path.indices()[0]);
        //let real_path = sortable_store
        //    .convert_path_to_child_path(path)
        //    .expect("Sorted path does not correspond to real path");
        //println!(
        //   "Clicked on sorted: {:?}, real: {:?}",
        //    path.indices(),
        //    real_path.indices()
        //);
    });
    tree.connect_cursor_changed(move |tree_view| {
        GLOBAL.with(move |global|{
            if let Some(ref ui) = *global.borrow(){
            let selection = tree_view.selection();
            if let Some((model, iter)) = selection.selected() {
                // Now getting back the values from the row corresponding to the
                // iterator `iter`.
                //
                // The `get_value` method do the conversion between the gtk type and Rust.
                ui.ui_label.set_text(&format!(
                    "Hello '{}' from rom {}",
                    model
                        .value(&iter, 1)
                        .get::<String>()
                        .expect("Treeview selection, column 1"),
                    model
                        .value(&iter, 0)
                        .get::<u32>()
                        .expect("Treeview selection, column 0"),
                ));
            }
            }
        });
    });

    // Adding the layout to the window.
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
