pub const TRANSMISSION_PROFILES: &[&str] = &[
    "Buggy", "Classic", "Electric_Hybrid", "Electric_Sportscar", "Electric_Supercar",
    "Electric_Hypercar", "Electric_HypercarALT", "Electric_HypercarALT2", "HotHatch",
    "HotRod", "Kei", "Muscle", "Offroad", "Race", "Rally", "Saloon", "SportsCar",
    "Supercar", "SUV", "TrackToy", "Truck", "TruckALT", "Van",
];

pub const TURBO_PROFILES: &[&str] = &[
    "Buggy", "Classic", "HotHatch", "HotRod", "Kei", "Muscle", "Offroad", "Race",
    "Rally", "Saloon", "SportsCar", "Supercar", "Hypercar", "SUV", "TrackToy",
];

pub const SUPER_CSC_PROFILES: &[&str] = TURBO_PROFILES;

pub const SUPER_DSC_PROFILES: &[&str] = &[
    "Buggy", "Classic", "HotHatch", "HotRod", "Kei", "Muscle", "Offroad", "Race",
    "Rally", "Saloon", "SportsCar", "Supercar", "Hypercar", "SUV", "TrackToy",
    "Truck", "Van", "MuscleALT",
];

pub const BURBLES_OPTIONS: &[&str] = &[
    "None",
    "ModernV8Muscle", "ModernV8Offroad", "ModernV8SportsCar", "ModernV8SUV", "ModernV8Saloon",
    "ModernV8Supercar", "ModernV8TrackToy", "ModernV8Truck", "ModernV8Van", "ClassicV8Muscle",
    "ClassicV8Offroad", "ClassicV8Race", "ClassicV8Saloon", "ClassicV8SportsCar", "ClassicV8Supercar",
    "ClassicV8TrackToy", "VintageV8Classic", "VintageV8HotRod", "VintageV8Muscle", "VintageV8Offroad",
    "VintageV8Race",
    "ModernV12Hypercar", "ModernV12Race", "ModernV12SUV", "ModernV12SportsCar", "ModernV12Supercar",
    "ModernV12TrackToy", "VintageV12Classic", "VintageV12Race",
    "ModernV10Muscle", "ModernV10Race", "ModernV10Saloon", "ModernV10Supercar", "ModernV10TrackToy",
    "ClassicV10Muscle", "ClassicV10Race", "ClassicV10Rally",
    "ModernV6Muscle", "ModernV6Offroad", "ModernV6Race", "ModernV6SUV", "ModernV6Saloon",
    "ModernV6SportsCar", "ModernV6Supercar", "ModernV6TrackToy", "ModernV6Truck", "ClassicV6Classic",
    "ClassicV6HotHatch", "ClassicV6Kei", "ClassicV6Muscle", "ClassicV6Rally", "ClassicV6SUV",
    "ClassicV6Saloon", "ClassicV6SportsCar", "ClassicV6Supercar", "ClassicV6TrackToy", "ClassicV6Van",
    "VintageV6Rally", "VintageV6SportsCar", "VintageV6TrackToy",
    "ModernI6Race", "ModernI6SUV", "ModernI6Saloon", "ModernI6SportsCar", "ModernI6TrackToy",
    "ModernI6Truck", "ClassicI6Race", "ClassicI6Saloon", "ClassicI6SportsCar", "ClassicI6Supercar",
    "VintageI6Classic", "VintageI6Muscle", "VintageI6Race", "VintageI6SportsCar", "VintageI6Truck",
    "ModernI5HotHatch", "ModernI5Saloon", "ModernI5SportsCar", "ModernI5TrackToy", "ClassicI5Rally",
    "ModernI4Buggy", "ModernI4Classic", "ModernI4HotHatch", "ModernI4Offroad", "ModernI4Race",
    "ModernI4Rally", "ModernI4Saloon", "ModernI4SportsCar", "ModernI4Supercar", "ModernI4TrackToy",
    "ModernI4Van", "ClassicI4Classic", "ClassicI4HotHatch", "ClassicI4Kei", "ClassicI4Race",
    "ClassicI4Rally", "ClassicI4Saloon", "ClassicI4SportsCar", "VintageI4Classic", "VintageI4HotHatch",
    "VintageI4Rally", "VintageI4SportsCar",
    "ModernI3Buggy", "ModernI3HotHatch", "ModernI3SportsCar", "ModernI3TrackToy", "ModernI2Offroad",
    "ModernI1Offroad", "ClassicI3Kei", "ClassicI3Race", "VintageI2Classic", "VintageI1Classic",
    "ModernF6Race", "ModernF6SportsCar", "ModernF6Supercar", "ModernF6TrackToy", "ClassicF6Race",
    "ClassicF6Rally", "ClassicF6SportsCar", "ClassicF6Supercar", "VintageF6Classic", "ModernF4Rally",
    "ModernF4Saloon", "ModernF4SportsCar", "ClassicF4Race", "ClassicF4Rally", "ClassicF4Saloon",
    "ClassicF4SportsCar", "ClassicF4Van", "VintageF4Buggy", "VintageF4Classic", "VintageF4Race",
    "ModernRotary2SportsCar", "ModernRotary3Race", "ClassicRotary2SportsCar", "VintageRotary2Classic",
    "VintageRotary2SportsCar", "VintageRotary4Race",
];

pub const BACKFIRE_ANTILAG_OPTIONS: &[&str] = &[
    "None", "I1", "I2", "I3", "I4", "I5", "I6", "I8", "V4", "V6", "V8", "V10", "V12",
    "F2", "F4", "F6", "F12", "W12", "W16", "Rally", "Rotary", "Scallop", "E0",
];

pub const BOV_OPTIONS: &[&str] = &[
    "None", "Stock", "SportsCar", "Supercar", "Hypercar", "TrackToy", "Buggy", "Kei",
    "HotHatch", "SUV", "Saloon", "Truck", "Van", "Classic", "HotRod", "Muscle",
    "Offroad", "Race", "Rally", "Scallop", "JDM", "Diesel",
];

pub const GEARCRACK_OPTIONS: &[&str] = &[
    "None", "Manual", "DCT", "DSG", "Sequential", "GearCrack_Sequential", "Race",
];

pub const LIMITER_OPTIONS: &[&str] = &[
    "None", "Limiter_Vintage", "Limiter_Classic", "Limiter_Modern",
];

pub const THROTTLE_OPTIONS: &[&str] = &[
    "None", "ThrottleBody_Vintage", "ThrottleBody_Classic", "ThrottleBody_Modern",
];

pub const ENGINE_BANK_OPTIONS: &[&str] = &[
    "GS_ModularCar", "GS_ModularCar_ALT",
];

pub const REVERB_FIELDS: &[&str] = &[
    "TrackReflectionDist", "NearDist", "FarDist", "NearDry", "NearWet",
    "FarDry", "FarWet", "DryLevelmB", "RoomLevelmB", "RoomHFLevelmB",
    "RoomRolloffFactor", "DecayTimeSec", "DecayHFRatio", "ReflectionsLevelmB",
    "ReflectionsDelaySec", "ReverbLevelmB", "ReverbDelaySec", "DiffusionPercent",
    "DensityPercent", "HFReferenceHz",
];

pub const GLOBAL_CATEGORIES: &[(&str, &str, &str)] = &[
    ("[Global Engine]",       "*_Eng.xml",          "Sets Channel volume/RPM for ALL engine synth files."),
    ("[Global Exhaust]",      "*_Exh.xml",          "Sets Channel volume/RPM for ALL exhaust synth files."),
    ("[Global Intake]",       "*_Int.xml",          "Sets Channel volume/RPM for ALL intake synth files."),
    ("[Global Transmission]", "*_Trn.xml",          "Sets Channel volume/RPM for ALL transmission synth files."),
    ("[Global Turbo]",        "*_Tbo.xml",          "Sets Channel volume/RPM for ALL turbocharger synth files."),
    ("[Global Supercharger]", "*_CSC.xml, *_DSC.xml","Sets Channel volume/RPM for ALL supercharger synth files."),
];

pub fn options_for_key(key: &str) -> &'static [&'static str] {
    if key.contains("Transmission") { TRANSMISSION_PROFILES }
    else if key.contains("Turbo") { TURBO_PROFILES }
    else if key.contains("SuperCSC") { SUPER_CSC_PROFILES }
    else if key.contains("SuperDSC") { SUPER_DSC_PROFILES }
    else if key.contains("Burbles") { BURBLES_OPTIONS }
    else if key.contains("Backfire") || key.contains("AntiLag") { BACKFIRE_ANTILAG_OPTIONS }
    else if key.contains("BOV") { BOV_OPTIONS }
    else if key.contains("GearCrack") { GEARCRACK_OPTIONS }
    else if key.contains("Limiter") { LIMITER_OPTIONS }
    else if key.contains("ThrottleBody") || key.contains("Throttle") { THROTTLE_OPTIONS }
    else if key.contains("EngineBank") { ENGINE_BANK_OPTIONS }
    else { &[] }
}
