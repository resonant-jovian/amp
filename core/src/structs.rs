use rust_decimal::Decimal;

#[derive(Debug, Default, Clone)]
pub struct AdressClean {
    pub coordinates: [Decimal; 2],
    pub postnummer: u16,
    pub adress: String,
    pub gata: String,
    pub gatunummer: String,
}

#[derive(Debug, Default, Clone)]
pub struct MiljoeDataClean {
    pub coordinates: [[Decimal; 2]; 2],
    pub info: String,
    pub tid: String,
    pub dag: u8,
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

#[derive(Debug, Default, Clone)]
pub struct Local {
    pub active: bool,
    pub postnummer: u16,
    pub adress: String,
    pub gata: String,
    pub gatunummer: String,
    pub info: String,
    pub tid: String,
    pub dag: u8,
}