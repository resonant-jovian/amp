use anyhow;
use arrow::{
    array::{
        BooleanArray, BooleanBuilder, Float64Array, Float64Builder, StringArray, StringBuilder,
        UInt8Array, UInt8Builder, UInt16Array, UInt16Builder,
    },
    datatypes::{DataType, Field, Schema},
    record_batch::RecordBatch,
};
use chrono::{Datelike, Duration, Local as ChronoLocal, NaiveDate, NaiveTime};
use get_fields::GetFields;
use iced::{
    Color, Element, Font, Length, Theme, theme,
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

const SCARLETFIRE: Color = Color::from_rgb(241. / 255., 46. / 255., 7. / 255.);
const SNOW: Color = Color::from_rgb(247. / 255., 246. / 255., 244. / 255.);
const ONYX: Color = Color::from_rgb(18. / 255., 19. / 255., 22. / 255.);
const HARVESTGOLD: Color = Color::from_rgb(224. / 255., 157. / 255., 49. / 255.);
const FORESTGREEN: Color = Color::from_rgb(26. / 255., 138. / 255., 56. / 255.);
const FORESTMOSS: Color = Color::from_rgb(108. / 255., 150. / 255., 17. / 255.);

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
    iced::application(Amp::default, Amp::update, Amp::view)
        .theme(Amp::theme)
        .default_font(Font::MONOSPACE)
        .run()
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
    //Better matching needed and fail response wanted
    local.adress == info.adress && local.postnummer == info.postnummer
}

fn add_one_month(date: NaiveDate) -> Option<NaiveDate> {
    let mut year = date.year();
    let mut month = date.month() + 1;

    if month == 13 {
        month = 1;
        year += 1;
    }

    NaiveDate::from_ymd_opt(year, month, date.day())
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
enum TimeBucket {
    Now,
    Within6Hours,
    Within1Day,
    Within1Month,
    Invalid,
}

fn bucket_for(info: &AdressInfo) -> TimeBucket {
    let remaining = match remaining_duration(info.dag, &info.tid) {
        Some(d) => d,
        None => return TimeBucket::Invalid,
    };

    // Treat everything ending within 6h as "Now"
    if remaining <= Duration::hours(4) {
        TimeBucket::Now
    } else if remaining <= Duration::hours(6) {
        TimeBucket::Within6Hours
    } else if remaining <= Duration::days(1) {
        TimeBucket::Within1Day
    } else if remaining <= Duration::days(31) {
        TimeBucket::Within1Month
    } else {
        TimeBucket::Invalid
    }
}

fn remaining_duration(dag: u8, tid: &str) -> Option<Duration> {
    let (_start, end) = parse_tid_interval(tid)?;

    let now = ChronoLocal::now().naive_local();
    let today = now.date();

    let this_month_date = NaiveDate::from_ymd_opt(today.year(), today.month(), dag.into())?;

    let this_end = this_month_date.and_time(end);

    // Case 1: still upcoming this month
    if this_end >= now {
        return Some(this_end - now);
    }

    // Case 2: already passed → roll to next month
    let next_month_date = add_one_month(this_month_date)?;
    let next_end = next_month_date.and_time(end);

    if next_end >= now {
        Some(next_end - now)
    } else {
        None
    }
}

fn render_bucket<'a>(
    title: &'a str,
    rows: &[(Duration, usize)],
    locals: &'a [Local],
    registry: &[AdressInfo],
) -> Element<'a, Message> {
    let content = rows.iter().map(|(_, idx)| {
        let local = &locals[*idx];

        let time_text = registry
            .iter()
            .find(|info| matches(local, info))
            .and_then(|info| remaining_until_interval(info.dag, &info.tid))
            .unwrap_or_else(|| ". . .".to_string());

        row![
            container(text(&local.adress)).padding(5).width(Length::Fill),
            container(text(time_text)).padding(5).width(Length::Shrink).style(container::dark),
        ]
        .spacing(20)
        .into()
    });

    container(column![
        container(column![
            container(text(title).size(24)).padding(5).width(Length::Fill),
            container(column(content).spacing(8)).padding(5).width(Length::Fill)
        ])
        .padding(5)
        .style(container::bordered_box),
    ])
    .padding(8).width(Length::Fill)
    .style(container::transparent)
    .into()
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
    let remaining = remaining_duration(dag, tid)?;

    let days = remaining.num_days();
    let hours = remaining.num_hours() % 24;
    let minutes = remaining.num_minutes() % 60;

    Some(format!("{}d {:02}h {:02}m", days, hours, minutes))
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
                } else {
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
                            .height(Length::Fixed(30.))
                            .style(button::danger),
                    ],
                    space().height(Length::Fixed(5.)),
                ]
                .into()
            },
        )));

        type BucketEntry = (Duration, usize); // (remaining, index into self.local)

        let mut buckets: BTreeMap<TimeBucket, Vec<BucketEntry>> = BTreeMap::new();

        for (idx, local) in self.local.iter().enumerate().filter(|(_, l)| l.active) {
            let Some(info) = self.registry.iter().find(|info| matches(local, info)) else {
                buckets
                    .entry(TimeBucket::Invalid)
                    .or_default()
                    .push((Duration::MAX, idx));
                continue;
            };

            if let Some(remaining) = remaining_duration(info.dag, &info.tid) {
                let bucket = bucket_for(info);
                buckets.entry(bucket).or_default().push((remaining, idx));
            } else {
                buckets
                    .entry(TimeBucket::Invalid)
                    .or_default()
                    .push((Duration::MAX, idx));
            }
        }

        for rows in buckets.values_mut() {
            rows.sort_by_key(|(remaining, _)| *remaining);
        }

        let now_rows = buckets
            .get(&TimeBucket::Now)
            .map(Vec::as_slice)
            .unwrap_or(&[]);
        let four_h_rows = buckets
            .get(&TimeBucket::Within6Hours)
            .map(Vec::as_slice)
            .unwrap_or(&[]);
        let one_d_rows = buckets
            .get(&TimeBucket::Within1Day)
            .map(Vec::as_slice)
            .unwrap_or(&[]);
        let one_m_rows = buckets
            .get(&TimeBucket::Within1Month)
            .map(Vec::as_slice)
            .unwrap_or(&[]);
        let invalid_rows = buckets
            .get(&TimeBucket::Invalid)
            .map(Vec::as_slice)
            .unwrap_or(&[]);

        let active_panel: Column<Message> = column![
            container(render_bucket(
                //Not actually working atm lol
                "Nu",
                now_rows,
                &self.local,
                &self.registry,
            ))
            .padding(5)
            .style(container::success),
            container(render_bucket(
                "Om mindre än 6h",
                four_h_rows,
                &self.local,
                &self.registry,
            ))
            .padding(5)
            .style(container::danger),
            container(render_bucket(
                "Inom 24h",
                one_d_rows,
                &self.local,
                &self.registry,
            ))
            .padding(5)
            .style(container::warning),
            container(render_bucket(
                "Inom 1 månad",
                one_m_rows,
                &self.local,
                &self.registry,
            ))
            .padding(5)
            .style(container::primary),
            container(render_bucket(
                "Ingen städning här",
                invalid_rows,
                &self.local,
                &self.registry,
            ))
            .padding(5)
            .style(container::dark),
        ]
        .into();

        container(scrollable(column![
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
            container(scrollable(active_panel),)
                .padding(10)
                .style(container::rounded_box),
        ]))
        .into()
    }

    fn theme(&self) -> Theme {
        Theme::custom(
            String::from("Custom"),
            theme::Palette {
                background: SNOW,
                primary: FORESTMOSS,
                text: ONYX,
                success: FORESTGREEN,
                warning: HARVESTGOLD,
                danger: SCARLETFIRE,
            },
        )
    }
}

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
