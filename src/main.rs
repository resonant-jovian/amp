use anyhow;
use arrow::{
    array::{
        BooleanArray, BooleanBuilder, Float64Array, Float64Builder, StringArray, StringBuilder,
        UInt8Array, UInt8Builder, UInt16Array, UInt16Builder,
    },
    datatypes::{DataType, Field, Schema},
    record_batch::RecordBatch,
};
use chrono::{Datelike, Local as ChronoLocal, NaiveDate, NaiveTime};
use get_fields::GetFields;
use iced::{
    Element, Length,
    widget::space::horizontal,
    widget::{
        Column, button, checkbox, column, container, row, scrollable, space, text, text_input,
    },
};
use parquet::{
    arrow::ArrowWriter,
    arrow::arrow_reader::ParquetRecordBatchReaderBuilder,
    file::properties::{EnabledStatistics, WriterProperties},
};
use std::{collections::BTreeMap, f64, fs::File, path::Path, sync::Arc};

#[derive(GetFields, Debug, Default)]
struct AdressInfo {
    debug_closest_line_id: u16,
    debug_distance: f64,
    coordinates: [f64; 2],
    postnummer: u16,
    adress: String,
    gata: String,
    gatunummer: String,
    info: String,
    tid: String,
    dag: u8,
}

#[derive(Debug, Default, Clone)]
struct Local {
    active: bool,
    debug_closest_line_id: u16,
    debug_distance: f64,
    coordinates: [f64; 2],
    postnummer: u16,
    adress: String,
    gata: String,
    gatunummer: String,
    info: String,
    tid: String,
    dag: u8,
}

#[derive(Debug, Default, Clone)]
struct LocalString {
    active: bool,
    debug_closest_line_id: u16,
    debug_distance: f64,
    coordinates: [f64; 2],
    postnummer: String,
    adress: String,
    gata: String,
    gatunummer: String,
    info: String,
    tid: String,
    dag: u8,
}

pub fn main() -> iced::Result {
    iced::run(Amp::update, Amp::view)
}

struct Amp {
    registry: Vec<AdressInfo>,
    local: Vec<Local>,
    input: LocalString,
}

#[derive(Debug, Clone)]
enum Message {
    GataChanged(String),
    GatunummerChanged(String),
    PostnummerChanged(String),
    AddAddressButtonPressed,
    RemoveAddressButtonPressed { index: usize },
    ToggleActive { index: usize, value: bool },
}

impl Default for Amp {
    fn default() -> Self {
        if Path::new("local.parquet").exists() == true {
            Self {
                input: LocalString::default(),
                local: read_local_parquet().unwrap_or_default(),
                registry: read_parquet().unwrap_or_default(),
            }
        } else {
            let schema = Arc::new(Schema::new(vec![
                Field::new("postnummer", DataType::UInt16, false), // groups row groups
                Field::new("gata", DataType::Utf8, false), // becomes column chunk inside group
                Field::new("adress", DataType::Utf8, false), // page-level data
                Field::new("gatunummer", DataType::Utf8, false), // page-level data
                Field::new("dag", DataType::UInt8, false),
                Field::new("tid", DataType::Utf8, false),
                Field::new("info", DataType::Utf8, false),
                Field::new("lat", DataType::Float64, false),
                Field::new("lon", DataType::Float64, false),
                Field::new("distance", DataType::Float64, false),
                Field::new("debug_closest_line_id", DataType::UInt16, false),
                Field::new("active", DataType::Boolean, false),
            ]));
            let file = File::create(Path::new("local.parquet")).unwrap();
            let writer =
                ArrowWriter::try_new(file, schema, None).expect("Failed to create ArrowWriter");
            writer.close().expect("Failed to close writer");
            Self {
                input: LocalString::default(),
                local: read_local_parquet().unwrap_or_default(),
                registry: read_parquet().unwrap_or_default(),
            }
        }
    }
}

fn matches(local: &Local, info: &AdressInfo) -> bool {
    //Improve this
    local.adress == info.adress && local.postnummer == info.postnummer
}

fn parse_tid_interval(tid: &str) -> Option<(NaiveTime, NaiveTime)> {
    let parts: Vec<_> = tid.split('-').collect();
    if parts.len() != 2 {
        return None;
    }

    let parse_hm = |s: &str| -> Option<NaiveTime> {
        let s = s.trim();
        if s.len() != 4 {
            return None;
        }
        let hour: u32 = s[0..2].parse().ok()?;
        let minute: u32 = s[2..4].parse().ok()?;
        NaiveTime::from_hms_opt(hour, minute, 0)
    };

    Some((parse_hm(parts[0])?, parse_hm(parts[1])?))
}

fn remaining_until_interval(dag: u8, tid: &str) -> Option<String> {
    let (start, end) = parse_tid_interval(tid)?;

    let today = ChronoLocal::now().date_naive();
    let year = today.year();
    let month = today.month();

    // NaiveDate for the relevant day
    let interval_date = NaiveDate::from_ymd_opt(year, month, dag.into())?;
    let end_datetime = interval_date.and_time(end);

    let now = ChronoLocal::now().naive_local();

    if now < end_datetime {
        let diff = end_datetime - now; // Duration
        let days = diff.num_days();
        let hours = diff.num_hours() % 24;
        let minutes = diff.num_minutes() % 60;
        Some(format!("{}d {:02}h {:02}m", days, hours, minutes))
    } else {
        Some("passed".to_string())
    }
}

impl Amp {
    fn update(&mut self, message: Message) {
        match message {
            Message::ToggleActive { index, value } => {
                if let Some(local) = self.local.get_mut(index) {
                    if local.active != value {
                        local.active = value;
                        write_parquet(self.local.clone());
                    }
                }
            }

            Message::AddAddressButtonPressed => {
                let mut new = self.input.clone();
                new.adress = format!("{} {}", new.gata.trim(), new.gatunummer.trim());
                if new.postnummer == "".to_string() {
                    self.local.push(Local {
                        active: new.active,
                        debug_closest_line_id: new.debug_closest_line_id,
                        debug_distance: new.debug_distance,
                        coordinates: new.coordinates,
                        postnummer: 0,
                        adress: new.adress,
                        gata: new.gata,
                        gatunummer: new.gatunummer,
                        info: new.info,
                        tid: new.tid,
                        dag: new.dag,
                    });
                }
                else {
                    self.local.push(Local {
                        active: new.active,
                        debug_closest_line_id: new.debug_closest_line_id,
                        debug_distance: new.debug_distance,
                        coordinates: new.coordinates,
                        postnummer: new
                            .postnummer
                            .trim()
                            .parse::<u16>()
                            .expect("Failed to parse postnummer"),
                        adress: new.adress,
                        gata: new.gata,
                        gatunummer: new.gatunummer,
                        info: new.info,
                        tid: new.tid,
                        dag: new.dag,
                    });
                }


                self.local.sort_by_key(|l| l.postnummer);

                write_parquet(self.local.clone());
            }

            Message::RemoveAddressButtonPressed { index } => {
                if self.local.len() < 2 {
                    self.local.clear();
                    let schema = Arc::new(Schema::new(vec![
                        Field::new("postnummer", DataType::UInt16, false), // groups row groups
                        Field::new("gata", DataType::Utf8, false), // becomes column chunk inside group
                        Field::new("adress", DataType::Utf8, false), // page-level data
                        Field::new("gatunummer", DataType::Utf8, false), // page-level data
                        Field::new("dag", DataType::UInt8, false),
                        Field::new("tid", DataType::Utf8, false),
                        Field::new("info", DataType::Utf8, false),
                        Field::new("lat", DataType::Float64, false),
                        Field::new("lon", DataType::Float64, false),
                        Field::new("distance", DataType::Float64, false),
                        Field::new("debug_closest_line_id", DataType::UInt16, false),
                        Field::new("active", DataType::Boolean, false),
                    ]));
                    let file = File::create(Path::new("local.parquet")).unwrap();
                    let writer = ArrowWriter::try_new(file, schema, None)
                        .expect("Failed to create ArrowWriter");
                    writer.close().expect("Failed to close writer");
                } else if index < self.local.len() {
                    self.local.remove(index);
                    write_parquet(self.local.clone());
                }
            }

            Message::GataChanged(v) => self.input.gata = v,
            Message::GatunummerChanged(v) => self.input.gatunummer = v,
            Message::PostnummerChanged(v) => match v {
                p => {
                    self.input.postnummer = p;
                }
            },
        }
    }

    fn view(&self) -> Element<'_, Message> {
        let input_row = row![
            text_input("Gata", &self.input.gata).on_input(Message::GataChanged),
            space().width(Length::Fixed(5.)),
            text_input("Gatunummer", &self.input.gatunummer).on_input(Message::GatunummerChanged),
            space().width(Length::Fixed(5.)),
            text_input("Postnummer", &self.input.postnummer.to_string())
                .on_input(Message::PostnummerChanged),
            space().width(Length::Fixed(5.)),
            button("+")
                .on_press(Message::AddAddressButtonPressed)
                .width(Length::Fixed(30.))
                .height(Length::Fixed(30.)),
        ];

        let adress_panel = scrollable(Column::from_iter(self.local.iter().enumerate().map(
            |(i, h)| {
                column![
                    row![
                        text(format!("{}, {}", h.adress.trim(), h.postnummer)),
                        horizontal(),
                        checkbox(h.active)
                            .on_toggle(move |value| Message::ToggleActive { index: i, value })
                            .size(30.),
                        space().width(Length::Fixed(5.)),
                        button("x")
                            .on_press(Message::RemoveAddressButtonPressed { index: i })
                            .width(Length::Fixed(30.))
                            .height(Length::Fixed(30.)),
                    ],
                    space().height(Length::Fixed(5.)),
                ]
                .into()
            },
        )));

        let active_rows = self.local.iter().filter(|l| l.active).map(|local| {
            let time_text = self
                .registry
                .iter()
                .find(|info| matches(local, info))
                .and_then(|info| remaining_until_interval(info.dag, &info.tid))
                .unwrap_or_else(|| ". . .".to_string());

            row![
                text(&local.adress).width(Length::Fill),
                text(time_text).width(Length::Shrink),
            ]
            .spacing(20)
            .into()
        });

        let active_panel = container(column![
            text("Aktiva adresser").size(20),
            column(active_rows).spacing(8),
        ])
        .padding(10)
        .style(container::rounded_box);

        container(column![
            container(column![
                container(input_row)
                    .padding(5)
                    .style(container::bordered_box),
                container(adress_panel)
                    .padding(5)
                    .style(container::bordered_box),
            ])
            .padding(10)
            .style(container::rounded_box),
            container(active_panel,)
                .padding(10)
                .style(container::rounded_box),
        ])
        .into()
    }
}
/*
#[derive(Debug, Clone)]
enum Message {
    GataChanged(String),
    GatunummerChanged(String),
    PostnummerChanged(String),
    AddAddressButtonPressed,

    ToggleActive { index: usize, value: bool },
}

#[derive(Debug)]
struct AppState {
    input: Local,
    locals: Vec<Local>,
    adress_info: Vec<AdressInfo>,
}
pub fn main() -> iced::Result {
    iced::run(update, view)
}

impl Default for AppState {
    fn default() -> Self {
        Self {
            input: Local::default(),
            locals: read_local_parquet().unwrap_or_default(),
            adress_info: read_parquet().unwrap_or_default(),
        }
    }
}

fn matches(local: &Local, info: &AdressInfo) -> bool {
    local.adress == info.adress && local.postnummer == info.postnummer
}

fn parse_tid_interval(tid: &str) -> Option<(NaiveTime, NaiveTime)> {
    let parts: Vec<_> = tid.split('-').collect();
    if parts.len() != 2 {
        return None;
    }

    let parse_hm = |s: &str| -> Option<NaiveTime> {
        let s = s.trim();
        if s.len() != 4 { return None; }
        let hour: u32 = s[0..2].parse().ok()?;
        let minute: u32 = s[2..4].parse().ok()?;
        NaiveTime::from_hms_opt(hour, minute, 0)
    };

    Some((parse_hm(parts[0])?, parse_hm(parts[1])?))
}

fn remaining_until_interval(dag: u8, tid: &str) -> Option<String> {
    let (start, end) = parse_tid_interval(tid)?;

    let today = ChronoLocal::now().date_naive();
    let year = today.year();
    let month = today.month();

    // NaiveDate for the relevant day
    let interval_date = NaiveDate::from_ymd_opt(year, month, dag.into())?;
    let end_datetime = interval_date.and_time(end);

    let now = ChronoLocal::now().naive_local();

    if now < end_datetime {
        let diff = end_datetime - now; // Duration
        let days = diff.num_days();
        let hours = diff.num_hours() % 24;
        let minutes = diff.num_minutes() % 60;
        Some(format!("{}d {:02}h {:02}m", days, hours, minutes))
    } else {
        Some("passed".to_string())
    }
}



fn view(state: &AppState) -> Element<'_, Message> {
    let adress_panel = state.locals.iter().enumerate().map(|(i, h)| {
        row![
            text(&h.adress),
            horizontal(),
            checkbox(h.active).on_toggle(move |value| Message::ToggleActive { index: i, value }),
        ]
        .padding(5)
        .into()
    });

    let active_rows = state.locals.iter()
        .filter(|l| l.active)
        .map(|local| {
            let time_text = state.adress_info
                .iter()
                .find(|info| matches(local, info))
                .and_then(|info| remaining_until_interval(info.dag, &info.tid))
                .unwrap_or_else(|| "—".to_string());

            row![
            text(&local.adress).width(Length::Fill),
            text(time_text).width(Length::Shrink),
        ]
                .spacing(20)
                .into()
        });

    let active_panel = container(column![
        text("Active addresses").size(20),
        column(active_rows).spacing(8),
    ])
    .padding(10)
    .style(container::rounded_box);

    container(column![
        container(row![
            text_input("Gata", &state.input.gata).on_input(Message::GataChanged),
            text_input("Gatunummer", &state.input.gatunummer).on_input(Message::GatunummerChanged),
            text_input("Postnummer", &state.input.postnummer.to_string())
                .on_input(Message::PostnummerChanged),
            button("+").on_press(Message::AddAddressButtonPressed),
        ])
        .padding(10),
        container(scrollable(Column::from_iter(adress_panel))).padding(10),
        active_panel
    ])
    .padding(10)
    .into()
}

fn update(state: &mut AppState, message: Message) {
    match message {
        Message::ToggleActive { index, value } => {
            if let Some(local) = state.locals.get_mut(index) {
                if local.active != value {
                    local.active = value;
                    write_parquet(state.locals.clone());
                }
            }
        }

        Message::AddAddressButtonPressed => {
            let mut new = state.input.clone();
            new.adress = format!("{} {}", new.gata.trim(), new.gatunummer.trim());

            state.locals.push(new);

            // 🔑 Match Parquet read order
            state.locals.sort_by_key(|l| l.postnummer);

            write_parquet(state.locals.clone());
        }

        Message::GataChanged(v) => state.input.gata = v,
        Message::GatunummerChanged(v) => state.input.gatunummer = v,
        Message::PostnummerChanged(v) => {
            if let Ok(p) = v.parse() {
                state.input.postnummer = p;
            }
        }
    }
}

 */

fn read_parquet() -> anyhow::Result<Vec<AdressInfo>> {
    // Open the Parquet file
    let file = File::open("adress_info.parquet").expect("Failed to open adress_info.parquet");

    // Build a Parquet reader
    let builder = ParquetRecordBatchReaderBuilder::try_new(file)
        .expect("Failed to create Parquet reader builder");

    let mut reader = builder
        .build()
        .expect("Failed to build Parquet record batch reader");

    let mut result = Vec::new();

    while let Some(batch) = reader.next().transpose()? {
        let batch: RecordBatch = batch;

        // Downcast each column to the correct type
        let debug_closest_line_id = batch
            .column(batch.schema().index_of("debug_closest_line_id")?)
            .as_any()
            .downcast_ref::<UInt16Array>()
            .expect("id column missing or wrong type");

        let debug_distance = batch
            .column(batch.schema().index_of("distance")?)
            .as_any()
            .downcast_ref::<Float64Array>()
            .expect("id column missing or wrong type");

        let lat = batch
            .column(batch.schema().index_of("lat")?)
            .as_any()
            .downcast_ref::<Float64Array>()
            .expect("lat_start column missing or wrong type");

        let lon = batch
            .column(batch.schema().index_of("lon")?)
            .as_any()
            .downcast_ref::<Float64Array>()
            .expect("lon_start column missing or wrong type");

        let postnummer = batch
            .column(batch.schema().index_of("postnummer")?)
            .as_any()
            .downcast_ref::<UInt16Array>()
            .expect("postnummer column missing or wrong type");

        let adress = batch
            .column(batch.schema().index_of("adress")?)
            .as_any()
            .downcast_ref::<StringArray>()
            .expect("adress column missing or wrong type");

        let gata = batch
            .column(batch.schema().index_of("gata")?)
            .as_any()
            .downcast_ref::<StringArray>()
            .expect("gata column missing or wrong type");

        let gatunummer = batch
            .column(batch.schema().index_of("gatunummer")?)
            .as_any()
            .downcast_ref::<StringArray>()
            .expect("gatunummer column missing or wrong type");

        let info = batch
            .column(batch.schema().index_of("info")?)
            .as_any()
            .downcast_ref::<StringArray>()
            .expect("info column missing or wrong type");

        let tid = batch
            .column(batch.schema().index_of("tid")?)
            .as_any()
            .downcast_ref::<StringArray>()
            .expect("tid column missing or wrong type");

        let dag = batch
            .column(batch.schema().index_of("dag")?)
            .as_any()
            .downcast_ref::<UInt8Array>()
            .expect("dag column missing or wrong type");

        // Convert rows to AdressInfo
        for i in 0..batch.num_rows() {
            let entry = AdressInfo {
                debug_closest_line_id: debug_closest_line_id.value(i),
                debug_distance: debug_distance.value(i),
                coordinates: [lat.value(i), lon.value(i)],
                postnummer: postnummer.value(i),
                adress: adress.value(i).to_string(),
                gata: gata.value(i).to_string(),
                gatunummer: gatunummer.value(i).to_string(),
                info: info.value(i).to_string(),
                tid: tid.value(i).to_string(),
                dag: dag.value(i),
            };
            result.push(entry);
        }
    }

    Ok(result)
}

fn read_local_parquet() -> anyhow::Result<Vec<Local>> {
    // Open the Parquet file
    let file = File::open("local.parquet").expect("Failed to open local.parquet");

    // Build a Parquet reader
    let builder = ParquetRecordBatchReaderBuilder::try_new(file)
        .expect("Failed to create Parquet reader builder");

    let mut reader = builder
        .build()
        .expect("Failed to build Parquet record batch reader");

    let mut result = Vec::new();

    while let Some(batch) = reader.next().transpose()? {
        let batch: RecordBatch = batch;

        // Downcast each column to the correct type
        let debug_closest_line_id = batch
            .column(batch.schema().index_of("debug_closest_line_id")?)
            .as_any()
            .downcast_ref::<UInt16Array>()
            .expect("id column missing or wrong type");

        let debug_distance = batch
            .column(batch.schema().index_of("distance")?)
            .as_any()
            .downcast_ref::<Float64Array>()
            .expect("distance column missing or wrong type");

        let lat = batch
            .column(batch.schema().index_of("lat")?)
            .as_any()
            .downcast_ref::<Float64Array>()
            .expect("lat column missing or wrong type");

        let lon = batch
            .column(batch.schema().index_of("lon")?)
            .as_any()
            .downcast_ref::<Float64Array>()
            .expect("lon column missing or wrong type");

        let postnummer = batch
            .column(batch.schema().index_of("postnummer")?)
            .as_any()
            .downcast_ref::<UInt16Array>()
            .expect("postnummer column missing or wrong type");

        let adress = batch
            .column(batch.schema().index_of("adress")?)
            .as_any()
            .downcast_ref::<StringArray>()
            .expect("adress column missing or wrong type");

        let gata = batch
            .column(batch.schema().index_of("gata")?)
            .as_any()
            .downcast_ref::<StringArray>()
            .expect("gata column missing or wrong type");

        let gatunummer = batch
            .column(batch.schema().index_of("gatunummer")?)
            .as_any()
            .downcast_ref::<StringArray>()
            .expect("gatunummer column missing or wrong type");

        let info = batch
            .column(batch.schema().index_of("info")?)
            .as_any()
            .downcast_ref::<StringArray>()
            .expect("info column missing or wrong type");

        let tid = batch
            .column(batch.schema().index_of("tid")?)
            .as_any()
            .downcast_ref::<StringArray>()
            .expect("tid column missing or wrong type");

        let dag = batch
            .column(batch.schema().index_of("dag")?)
            .as_any()
            .downcast_ref::<UInt8Array>()
            .expect("dag column missing or wrong type");

        let active = batch
            .column(batch.schema().index_of("active")?)
            .as_any()
            .downcast_ref::<BooleanArray>()
            .expect("active column missing or wrong type");

        // Convert rows to AdressInfo
        for i in 0..batch.num_rows() {
            let entry = Local {
                debug_closest_line_id: debug_closest_line_id.value(i),
                debug_distance: debug_distance.value(i),
                coordinates: [lat.value(i), lon.value(i)],
                postnummer: postnummer.value(i),
                adress: adress.value(i).to_string(),
                gata: gata.value(i).to_string(),
                gatunummer: gatunummer.value(i).to_string(),
                info: info.value(i).to_string(),
                tid: tid.value(i).to_string(),
                dag: dag.value(i),
                active: active.value(i),
            };
            result.push(entry);
        }
    }

    Ok(result)
}

fn write_parquet(data: Vec<Local>) -> Option<String> {
    if data.is_empty() {
        return None;
    }
    // -------------------------------
    // 1. Define schema
    // -------------------------------
    let schema = Arc::new(Schema::new(vec![
        Field::new("postnummer", DataType::UInt16, false), // groups row groups
        Field::new("gata", DataType::Utf8, false),         // becomes column chunk inside group
        Field::new("adress", DataType::Utf8, false),       // page-level data
        Field::new("gatunummer", DataType::Utf8, false),   // page-level data
        Field::new("dag", DataType::UInt8, false),
        Field::new("tid", DataType::Utf8, false),
        Field::new("info", DataType::Utf8, false),
        Field::new("lat", DataType::Float64, false),
        Field::new("lon", DataType::Float64, false),
        Field::new("distance", DataType::Float64, false),
        Field::new("debug_closest_line_id", DataType::UInt16, false),
        Field::new("active", DataType::Boolean, false),
    ]));

    // --------------------------------------------------------------------
    // 2. Group input by postal code → row groups
    // --------------------------------------------------------------------
    let mut grouped: BTreeMap<u16, Vec<Local>> = BTreeMap::new();

    for d in data {
        grouped.entry(d.postnummer).or_insert_with(Vec::new).push(d);
    }

    // -------------------------------
    // 3. Parquet writer
    // -------------------------------
    let path = "local.parquet".to_string();
    let file = File::create(&path).expect("Failed to create file");
    let props = WriterProperties::builder()
        .set_statistics_enabled(EnabledStatistics::None)
        .build();
    let mut writer = ArrowWriter::try_new(file, schema.clone(), Some(props))
        .expect("Failed to create ArrowWriter");

    // -------------------------------
    // 4. Write each dag group as a row group
    // -------------------------------
    for (postnummer, rows) in grouped {
        let mut post_builder = UInt16Builder::new();
        let mut gata_builder = StringBuilder::new();
        let mut adress_builder = StringBuilder::new();
        let mut nr_builder = StringBuilder::new();
        let mut dag_builder = UInt8Builder::new();
        let mut tid_builder = StringBuilder::new();
        let mut info_builder = StringBuilder::new();
        let mut lat_builder = Float64Builder::new();
        let mut lon_builder = Float64Builder::new();
        let mut distance_builder = Float64Builder::new();
        let mut id_builder = UInt16Builder::new();
        let mut active_builder = BooleanBuilder::new();

        for r in rows {
            post_builder.append_value(postnummer);
            gata_builder.append_value(&r.gata);
            adress_builder.append_value(&r.adress);
            nr_builder.append_value(&r.gatunummer);
            dag_builder.append_value(r.dag);
            tid_builder.append_value(&r.tid);
            info_builder.append_value(&r.info);
            lat_builder.append_value(r.coordinates[0]);
            lon_builder.append_value(r.coordinates[1]);
            distance_builder.append_value(r.debug_distance);
            id_builder.append_value(r.debug_closest_line_id);
            active_builder.append_value(r.active);
        }

        let batch = RecordBatch::try_new(
            schema.clone(),
            vec![
                Arc::new(post_builder.finish()),
                Arc::new(gata_builder.finish()),
                Arc::new(adress_builder.finish()),
                Arc::new(nr_builder.finish()),
                Arc::new(dag_builder.finish()),
                Arc::new(tid_builder.finish()),
                Arc::new(info_builder.finish()),
                Arc::new(lat_builder.finish()),
                Arc::new(lon_builder.finish()),
                Arc::new(distance_builder.finish()),
                Arc::new(id_builder.finish()),
                Arc::new(active_builder.finish()),
            ],
        )
        .expect("Failed to create record batch");

        writer.write(&batch).expect("Failed to write batch"); // each write() = one row group
    }

    writer.close().expect("Writer failed to close");

    Some(path)
}
