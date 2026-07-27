use todo::{
    persist::{binary::TodosBin, jsonfile::TodosJsonDB},
    Todos,
};

#[test]
fn independent_clients_do_not_lose_each_others_changes() {
    let directory = tempfile::tempdir().unwrap();
    let filename = directory.path().join("todos.bin");
    let first = Todos::new(TodosBin::at(filename.clone()));
    let second = Todos::new(TodosBin::at(filename));

    first.add_message("first").unwrap();
    second.add_message("second").unwrap();

    let messages = first
        .get_all()
        .unwrap()
        .into_iter()
        .map(|todo| todo.message)
        .collect::<Vec<_>>();

    assert_eq!(messages, ["first", "second"]);
}

#[test]
fn json_storage_starts_empty_and_persists_mutations() {
    let directory = tempfile::tempdir().unwrap();
    let filename = directory.path().join("todos.json");
    let todos = Todos::new(TodosJsonDB::at(filename));

    todos.add_message("from json").unwrap();

    assert_eq!(todos.get_all().unwrap()[0].message, "from json");
}

#[test]
fn concurrent_clients_preserve_every_mutation() {
    let directory = tempfile::tempdir().unwrap();
    let filename = directory.path().join("concurrent.bin");
    let clients = (0..16)
        .map(|index| {
            let filename = filename.clone();
            std::thread::spawn(move || {
                Todos::new(TodosBin::at(filename))
                    .add_message(&format!("todo {index}"))
                    .unwrap();
            })
        })
        .collect::<Vec<_>>();

    for client in clients {
        client.join().unwrap();
    }

    assert_eq!(
        Todos::new(TodosBin::at(filename)).get_all().unwrap().len(),
        16
    );
}
