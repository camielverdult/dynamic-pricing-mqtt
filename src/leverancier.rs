use serde::Serialize;

#[derive(Debug, Clone, Copy)]
pub enum Leverancier {
    Generic = 0,
    All_in_power = 4,
    ANWB_Energie = 3,
    BudgetEnergie = 15,
    CoolblueEnergie = 10,
    DeltaEnergie = 22,
    easyEnergy = 5,
    Eneco = 17,
    EnergieVanOns = 6,
    Energiedirect = 16,
    Energiek = 21,
    EnergyZero = 7,
    Engie = 23,
    Essent = 20,
    FrankEnergie = 8,
    GroeneStroomLokaal = 9,
    NextEnergy = 11,
    Oxxio = 19,
    Tibber = 1,
    Vandebron = 14,
    Vattenfall = 18,
    Vrijopnaam = 12,
    Zonneplan = 2,
}
