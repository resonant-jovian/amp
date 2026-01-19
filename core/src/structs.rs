#[derive(Debug, Clone)]
pub struct AdressClean {
    pub coordinates: [f64; 2],
    pub postnummer: u16,
    pub adress: String,
    pub gata: String,
    pub gatunummer: String,
}

#[derive(Debug, Clone)]
pub struct MiljoeDataClean {
    pub coordinates: [[f64; 2]; 2],
    pub info: String,
    pub tid: String,
    pub dag: u8,
    pub id: u16,
}

#[derive(Debug, Default)]
pub struct AdressInfo {
    pub relevant: bool,
    pub postnummer: u16,
    pub adress: String,
    pub gata: String,
    pub gatunummer: String,
    pub info: String,
    pub tid: String,
    pub dag: u8,
}