use iced::widget::{button, column, container, row, scrollable, text, text_input};
use iced::Element;
//use rusty_math::knn::{KNNeighborsClassifier, KNNeighborsRegressor};
use serde::Serialize;
use std::fs::File;
use std::io::Write;
use get_fields::GetFields;

#[derive(Default, Serialize)]
struct AdressInit {
    gata: String,
    gatunummer: String,
    postnummer: String,
}
#[derive(Default, Serialize)]
struct AdressExit {
    gata: String,
    gatunummer: String,
    postnummer: String,
    aktiv: bool,
}

#[derive(GetFields, Debug, Default)]
struct AdressInfo {
    debug_closest_line_id: u16,
    coordinates: [f64; 2],
    postnummer: u16,
    adress: String,
    gata: String,
    gatunummer: String,
    info: String,
    tid: String,
    dag: u8,
}


#[derive(Debug, Clone)]
enum Message {
    GataChanged(String),
    GatunummerChanged(String),
    PostnummerChanged(String),
    AddAddressButtonPressed,
}

pub fn main() -> iced::Result {
    iced::run(update, view)
}

fn view(state: &AdressInit) -> Element<'_, Message> {
    let index: Vec<&str>  = AdressInfo::get_fields.to_owned();
    let holder = read_parquet();
    column![
        container(row![
            text_input("Gata", &state.gata).on_input(Message::GataChanged),
            text_input("Gatunummer", &state.gatunummer).on_input(Message::GatunummerChanged),
            text_input("Postnummer", &state.postnummer).on_input(Message::PostnummerChanged),
            button("+").on_press(Message::AddAddressButtonPressed),
        ])
        .padding(10)
        .style(container::rounded_box),


        container(scrollable(

            text(holder.first().unwrap().adress.clone())

        ))

        .padding(10)
        .style(container::rounded_box)
    ]
        .into()
}


fn update(state: &mut AdressInit, message: Message) {
    match message {
        Message::GataChanged(gata) => {
            state.gata = gata;
        }
        Message::GatunummerChanged(gatunummer) => {
            state.gatunummer = gatunummer;
        }
        Message::PostnummerChanged(postnummer) => {
            state.postnummer = postnummer;
        }
        Message::AddAddressButtonPressed => {
            write_json(state).unwrap(); //Add to JSON list and write //Remove unwrap
        }
    }
}

fn write_json(data: &mut AdressInit) -> std::io::Result<()> {
    let structure = AdressExit {
        gata: data.gata.clone(),
        gatunummer: data.gatunummer.clone(),
        postnummer: data.postnummer.clone(),
        aktiv: true,
    };
    let json_data = serde_json::to_string_pretty(&structure)?;
    let mut file = File::create("adresser.json")?;
    file.write_all(json_data.as_bytes())?;
    Ok(())
}

fn read_parquet() -> Vec<AdressInfo> {

    Default::default()
}

/*

fn knnc_eu() { //-> Vec<i32>
    let mut knn = KNNeighborsClassifier::new(2, "euclidean".to_string(), None);
    let x_train = vec![vec![55.567676, 13.011854], vec![55.567637, 13.012091]];
    let y_train = vec![55.5568509, 12.9182615];
    knn.fit(&x_train, &y_train);
    let x = vec![55.5568509, 12.9182615];
    let y = knn.knearest_neighbors(&x);
    let params = knn.get_params();
    println!("KNNC euclidean");
    println!("{:#?}", y);
    println!("{:#?}", params);
}
fn knnc_min() { //-> Vec<i32>
    let mut knn = KNNeighborsClassifier::new(2, "minkowski".to_string(), Some(50));
    let x_train = vec![vec![55.567676, 13.011854], vec![55.567637, 13.012091]];
    let y_train = vec![55.5568509, 12.9182615];
    knn.fit(&x_train, &y_train);
    let x = vec![55.5568509, 12.9182615];
    let y = knn.knearest_neighbors(&x);
    let params = knn.get_params();
    println!("KNNC minkowski");
    println!("{:#?}", y);
    println!("{:#?}", params);
}
fn knnc_cos() { //-> Vec<i32>
    let mut knn = KNNeighborsClassifier::new(2, "cosine".to_string(), None);
    let x_train = vec![vec![55.567676, 13.011854], vec![55.567637, 13.012091]];
    let y_train = vec![55.5568509, 12.9182615];
    knn.fit(&x_train, &y_train);
    let x = vec![55.5568509, 12.9182615];
    let y = knn.knearest_neighbors(&x);
    let params = knn.get_params();
    println!("KNNC cosine");
    println!("{:#?}", y);
    println!("{:#?}", params);
}
fn knnc_man() { //-> Vec<i32>
    let mut knn = KNNeighborsClassifier::new(2, "manhattan".to_string(), None);
    let x_train = vec![vec![55.567676, 13.011854], vec![55.567637, 13.012091]];
    let y_train = vec![55.5568509, 12.9182615];
    knn.fit(&x_train, &y_train);
    let x = vec![55.5568509, 12.9182615];
    let y = knn.knearest_neighbors(&x);
    let params = knn.get_params();
    println!("KNNC manhattan");
    println!("{:#?}", y);
    println!("{:#?}", params);
}

fn knnr_eu() { //-> Vec<i32>
    let mut knn = KNNeighborsRegressor::new(2, "euclidean".to_string(), None);
    let x_train = vec![vec![55.567676, 13.011854], vec![55.567637, 13.012091]];
    let y_train = vec![55.5568509, 12.9182615];
    knn.fit(&x_train, &y_train);
    let x = vec![55.5568509, 12.9182615];
    let y = knn.knearest_neighbors(&x);
    let params = knn.get_params();
    println!("KNNR euclidean");
    println!("{:#?}", y);
    println!("{:#?}", params);
}
fn knnr_min() { //-> Vec<i32>
    let mut knn = KNNeighborsRegressor::new(2, "minkowski".to_string(), Some(50));
    let x_train = vec![vec![55.567676, 13.011854], vec![55.567637, 13.012091]];
    let y_train = vec![55.5568509, 12.9182615];
    knn.fit(&x_train, &y_train);
    let x = vec![55.5568509, 12.9182615];
    let y = knn.knearest_neighbors(&x);
    let params = knn.get_params();
    println!("KNNR minkowski");
    println!("{:#?}", y);
    println!("{:#?}", params);
}
fn knnr_cos() { //-> Vec<i32>
    let mut knn = KNNeighborsRegressor::new(2, "cosine".to_string(), None);
    let x_train = vec![vec![55.567676, 13.011854], vec![55.567637, 13.012091]];
    let y_train = vec![55.5568509, 12.9182615];
    knn.fit(&x_train, &y_train);
    let x = vec![55.5568509, 12.9182615];
    let y = knn.knearest_neighbors(&x);
    let params = knn.get_params();
    println!("KNNR cosine");
    println!("{:#?}", y);
    println!("{:#?}", params);
}
fn knnr_man() { //-> Vec<i32>
    let mut knn = KNNeighborsRegressor::new(2, "manhattan".to_string(), None);
    let x_train = vec![vec![55.567676, 13.011854], vec![55.567637, 13.012091]];
    let y_train = vec![55.5568509, 12.9182615];
    knn.fit(&x_train, &y_train);
    let x = vec![55.5568509, 12.9182615];
    let y = knn.knearest_neighbors(&x);
    let params = knn.get_params();
    println!("KNNR manhattan");
    println!("{:#?}", y);
    println!("{:#?}", params);
}

 */