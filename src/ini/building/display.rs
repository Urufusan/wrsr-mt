use std::fmt::{Formatter, Error, Display};
use std::io::{Write};

use super::{BuildingType,
            BuildingSubtype,
            StorageCargoType,
            ConstructionPhase,
            ConstructionAutoCost,
            ResourceType,
            ParticleType,
            Token,
            StrValue,
            QuotedStringParam,
            IdStringParam,
            Point3f,
            Rect,
           };


type IOResult = Result<(), std::io::Error>;


impl Token<'_> {
    pub fn serialize_token<W: Write>(&self, mut wr: W) -> IOResult {
        match self {
            Self::NameStr(p)                       => write!(wr, "{} {}", Self::NAME_STR, p),
            Self::Name(p)                          => write!(wr, "{} {}", Self::NAME, p),
            Self::BuildingType(p)                  => write!(wr, "TYPE_{}", p),
            Self::BuildingSubtype(p)               => write!(wr, "SUBTYPE_{}", p),
            Self::CivilBuilding                    => write!(wr, "{}", Self::CIVIL_BUILDING),
            Self::QualityOfLiving(x)               => write!(wr, "{} {}", Self::QUALITY_OF_LIVING, x),

            Self::WorkersNeeded(x)                 => write!(wr, "{} {}", Self::WORKERS_NEEDED, x),
            Self::ProfessorsNeeded(x)              => write!(wr, "{} {}", Self::PROFESSORS_NEEDED, x),
            Self::CitizenAbleServe(x)              => write!(wr, "{} {}", Self::CITIZEN_ABLE_SERVE, x),

            Self::Storage((t, x))                  => write!(wr, "{} {} {}", Self::STORAGE, t, x),

            Self::ConnectionPedestrian((x, y))     => write!(wr, "{}\r\n{}\r\n{}", Self::CONNECTION_PEDESTRIAN, x, y),
            Self::ConnectionRoad((x, y))           => write!(wr, "{}\r\n{}\r\n{}", Self::CONNECTION_ROAD, x, y),
            Self::ConnectionRoadDead(x)            => write!(wr, "{}\r\n{}", Self::CONNECTION_ROAD_DEAD, x),
            Self::ConnectionsRoadDeadSquare(
                Rect { x1, z1, x2, z2 })           => write!(wr, "{}\r\n{} {}\r\n{} {}", Self::CONNECTIONS_ROAD_DEAD_SQUARE, x1, z1, x2, z2),

            Self::Particle((t, x, a, s))           => write!(wr, "{} {} {} {} {}", Self::PARTICLE, t, x, a, s),
            Self::TextCaption((x, y))              => write!(wr, "{}\r\n{}\r\n{}", Self::TEXT_CAPTION, x, y),

            Self::CostWork((t, x))                 => write!(wr, "{} {} {}", Self::COST_WORK, t, x),
            Self::CostWorkBuildingNode(n)          => write!(wr, "{} {}", Self::COST_WORK_BUILDING_NODE, n),
            Self::CostResource((t, x))             => write!(wr, "{} {} {}", Self::COST_RESOURCE, t, x),
            Self::CostResourceAuto((t, x))         => write!(wr, "{} {} {}", Self::COST_RESOURCE_AUTO, t, x),
            Self::CostWorkVehicleStation((x, y))   => write!(wr, "{}\r\n{}\r\n{}", Self::COST_WORK_VEHICLE_STATION, x, y),
            Self::CostWorkVehicleStationNode(p)    => write!(wr, "{} {}", Self::COST_WORK_VEHICLE_STATION_NODE, p),
        }
    }
}

impl Display for Token<'_> {
    fn fmt(&self, f: &mut Formatter) -> Result<(), Error> {
        write!(f, "$")?;
        match self {
            Self::NameStr(p)                       => write!(f, "{} {}", Self::NAME_STR, p),
            Self::Name(p)                          => write!(f, "{} {}", Self::NAME, p),
            Self::BuildingType(p)                  => write!(f, "TYPE_{}", p),
            Self::BuildingSubtype(p)               => write!(f, "SUBTYPE_{}", p),
            Self::CivilBuilding                    => write!(f, "{}", Self::CIVIL_BUILDING),
            Self::QualityOfLiving(x)               => write!(f, "{} {}", Self::QUALITY_OF_LIVING, x),

            Self::WorkersNeeded(x)                 => write!(f, "{} {}", Self::WORKERS_NEEDED, x),
            Self::ProfessorsNeeded(x)              => write!(f, "{} {}", Self::PROFESSORS_NEEDED, x),
            Self::CitizenAbleServe(x)              => write!(f, "{} {}", Self::CITIZEN_ABLE_SERVE, x),

            Self::Storage((t, x))                  => write!(f, "{} {} {}", Self::STORAGE, t, x),

            Self::ConnectionPedestrian((x, y))     => write!(f, "{} {} {}", Self::CONNECTION_PEDESTRIAN, x, y),
            Self::ConnectionRoad((x, y))           => write!(f, "{} {} {}", Self::CONNECTION_ROAD, x, y),
            Self::ConnectionRoadDead(x)            => write!(f, "{} {}", Self::CONNECTION_ROAD_DEAD, x),
            Self::ConnectionsRoadDeadSquare(rect)  => write!(f, "{} {}", Self::CONNECTIONS_ROAD_DEAD_SQUARE, rect),

            Self::Particle((t, x, a, s))           => write!(f, "{} {} {} {} {}", Self::PARTICLE, t, x, a, s),
            Self::TextCaption((x, y))              => write!(f, "{} {} {}", Self::TEXT_CAPTION, x, y),

            Self::CostWork((t, x))                 => write!(f, "{} {} {}", Self::COST_WORK, t, x),
            Self::CostWorkBuildingNode(n)          => write!(f, "{} {}", Self::COST_WORK_BUILDING_NODE, n),
            Self::CostResource((t, x))             => write!(f, "{} {} {}", Self::COST_RESOURCE, t, x),
            Self::CostResourceAuto((t, x))         => write!(f, "{} {} {}", Self::COST_RESOURCE_AUTO, t, x),
            Self::CostWorkVehicleStation((x, y))   => write!(f, "{} {} {}", Self::COST_WORK_VEHICLE_STATION, x, y),
            Self::CostWorkVehicleStationNode(p)    => write!(f, "{} {}", Self::COST_WORK_VEHICLE_STATION_NODE, p),
        }
    }
}


impl Display for BuildingType {
    fn fmt(&self, f: &mut Formatter) -> Result<(), Error> {
        let s = match self {
            Self::AirplaneGate           => Self::TYPE_AIRPLANE_GATE,
            Self::AirplaneParking        => Self::TYPE_AIRPLANE_PARKING,
            Self::AirplaneTower          => Self::TYPE_AIRPLANE_TOWER,
            Self::Attraction             => Self::TYPE_ATTRACTION,
            Self::Broadcast              => Self::TYPE_BROADCAST,
            Self::CarDealer              => Self::TYPE_CAR_DEALER,
            Self::CargoStation           => Self::TYPE_CARGO_STATION,
            Self::Church                 => Self::TYPE_CHURCH,
            Self::Cityhall               => Self::TYPE_CITYHALL,
            Self::ConstructionOffice     => Self::TYPE_CONSTRUCTION_OFFICE,
            Self::ConstructionOfficeRail => Self::TYPE_CONSTRUCTION_OFFICE_RAIL,
            Self::ContainerFacility      => Self::TYPE_CONTAINER_FACILITY,
            Self::CoolingTower           => Self::TYPE_COOLING_TOWER,
            Self::Customhouse            => Self::TYPE_CUSTOMHOUSE,
            Self::DistributionOffice     => Self::TYPE_DISTRIBUTION_OFFICE,
            Self::ElectricExport         => Self::TYPE_ELETRIC_EXPORT,
            Self::ElectricImport         => Self::TYPE_ELETRIC_IMPORT,
            Self::Engine                 => Self::TYPE_ENGINE,
            Self::Factory                => Self::TYPE_FACTORY,
            Self::Farm                   => Self::TYPE_FARM,
            Self::Field                  => Self::TYPE_FIELD,
            Self::Firestation            => Self::TYPE_FIRESTATION,
            Self::ForkliftGarage         => Self::TYPE_FORKLIFT_GARAGE,
            Self::GarbageOffice          => Self::TYPE_GARBAGE_OFFICE,
            Self::GasStation             => Self::TYPE_GAS_STATION,
            Self::HeatingEndstation      => Self::TYPE_HEATING_ENDSTATION,
            Self::HeatingPlant           => Self::TYPE_HEATING_PLANT,
            Self::HeatingSwitch          => Self::TYPE_HEATING_SWITCH,
            Self::Hospital               => Self::TYPE_HOSPITAL,
            Self::Hotel                  => Self::TYPE_HOTEL,
            Self::Kindergarten           => Self::TYPE_KINDERGARTEN,
            Self::Kino                   => Self::TYPE_KINO,
            Self::Living                 => Self::TYPE_LIVING,
            Self::MineBauxite            => Self::TYPE_MINE_BAUXITE,
            Self::MineCoal               => Self::TYPE_MINE_COAL,
            Self::MineGravel             => Self::TYPE_MINE_GRAVEL,
            Self::MineIron               => Self::TYPE_MINE_IRON,
            Self::MineOil                => Self::TYPE_MINE_OIL,
            Self::MineUranium            => Self::TYPE_MINE_URANIUM,
            Self::MineWood               => Self::TYPE_MINE_WOOD,
            Self::Monument               => Self::TYPE_MONUMENT,
            Self::Parking                => Self::TYPE_PARKING,
            Self::PassangerStation       => Self::TYPE_PASSANGER_STATION,
            Self::PedestrianBridge       => Self::TYPE_PEDESTRIAN_BRIDGE,
            Self::PoliceStation          => Self::TYPE_POLICE_STATION,
            Self::PollutionMeter         => Self::TYPE_POLLUTION_METER,
            Self::Powerplant             => Self::TYPE_POWERPLANT,
            Self::ProductionLine         => Self::TYPE_PRODUCTION_LINE,
            Self::Pub                    => Self::TYPE_PUB,
            Self::RailTrafo              => Self::TYPE_RAIL_TRAFO,
            Self::Raildepo               => Self::TYPE_RAILDEPO,
            Self::Roaddepo               => Self::TYPE_ROADDEPO,
            Self::School                 => Self::TYPE_SCHOOL,
            Self::ShipDock               => Self::TYPE_SHIP_DOCK,
            Self::Shop                   => Self::TYPE_SHOP,
            Self::Sport                  => Self::TYPE_SPORT,
            Self::Storage                => Self::TYPE_STORAGE,
            Self::Substation             => Self::TYPE_SUBSTATION,
            Self::Transformator          => Self::TYPE_TRANSFORMATOR,
            Self::University             => Self::TYPE_UNIVERSITY,
        };

        write!(f, "{}", s)
    }
}


impl Display for BuildingSubtype {
    fn fmt(&self, f: &mut Formatter) -> Result<(), Error> {
        let s = match self {
            Self::Aircustom        => Self::SUBTYPE_AIRCUSTOM,
            Self::Airplane         => Self::SUBTYPE_AIRPLANE,
            Self::Cableway         => Self::SUBTYPE_CABLEWAY,
            Self::Hostel           => Self::SUBTYPE_HOSTEL,
            Self::Medical          => Self::SUBTYPE_MEDICAL,
            Self::Radio            => Self::SUBTYPE_RADIO,
            Self::Rail             => Self::SUBTYPE_RAIL,
            Self::Restaurant       => Self::SUBTYPE_RESTAURANT,
            Self::Road             => Self::SUBTYPE_ROAD,
            Self::Ship             => Self::SUBTYPE_SHIP,
            Self::Soviet           => Self::SUBTYPE_SOVIET,
            Self::SpaceForVehicles => Self::SUBTYPE_SPACE_FOR_VEHICLES,
            Self::Technical        => Self::SUBTYPE_TECHNICAL,
            Self::Television       => Self::SUBTYPE_TELEVISION,
            Self::Trolleybus       => Self::SUBTYPE_TROLLEYBUS,
        };

        write!(f, "{}", s)
    }
}


impl Display for StorageCargoType {
    fn fmt(&self, f: &mut Formatter) -> Result<(), Error> {
        let s = match self {
            Self::Passanger => Self::PASSANGER,
            Self::Cement    => Self::CEMENT,
            Self::Covered   => Self::COVERED,
            Self::Gravel    => Self::GRAVEL,
            Self::Oil       => Self::OIL,
            Self::Open      => Self::OPEN,
            Self::Cooler    => Self::COOLER,
            Self::Concrete  => Self::CONCRETE,
            Self::Livestock => Self::LIVESTOCK,
            Self::General   => Self::GENERAL
        };

        write!(f, "{}", s)
    }
}


impl Display for ConstructionAutoCost {
    fn fmt(&self, f: &mut Formatter) -> Result<(), Error> {
        let s = match self {
            Self::Ground        => Self::GROUND,
            Self::GroundAsphalt => Self::GROUND_ASPHALT,
            Self::WallConcrete  => Self::WALL_CONCRETE,
            Self::WallPanels    => Self::WALL_PANELS,
            Self::WallBrick     => Self::WALL_BRICK,
            Self::WallSteel     => Self::WALL_STEEL,
            Self::WallWood      => Self::WALL_WOOD,
            Self::TechSteel     => Self::TECH_STEEL,
            Self::ElectroSteel  => Self::ELECTRO_STEEL,
            Self::RoofWoodBrick => Self::ROOF_WOOD_BRICK,
            Self::RoofSteel     => Self::ROOF_STEEL,
            Self::RoofWoodSteel => Self::ROOF_WOOD_STEEL,
        };

        write!(f, "{}", s)
    }
}


impl Display for ConstructionPhase {
    fn fmt(&self, f: &mut Formatter) -> Result<(), Error> {
        let s = match self {
            Self::Groundworks     => Self::GROUNDWORKS,
            Self::BoardsLaying    => Self::BOARDS_LAYING,
            Self::BricksLaying    => Self::BRICKS_LAYING,
            Self::SkeletonCasting => Self::SKELETON_CASTING,
            Self::SteelLaying     => Self::STEEL_LAYING,
            Self::PanelsLaying    => Self::PANELS_LAYING,
            Self::RooftopBuilding => Self::ROOFTOP_BUILDING,
            Self::WireLaying      => Self::WIRE_LAYING,
            Self::Tunneling       => Self::TUNNELING,
        };

        write!(f, "{}", s)
    }
}


impl Display for ResourceType {
    fn fmt(&self, f: &mut Formatter) -> Result<(), Error> {
        let s = match self {
            Self::Alcohol           => Self::ALCOHOL,
            Self::Alumina           => Self::ALUMINA,
            Self::Aluminium         => Self::ALUMINIUM,
            Self::Asphalt           => Self::ASPHALT,
            Self::Bauxite           => Self::BAUXITE,
            Self::Boards            => Self::BOARDS,
            Self::Bricks            => Self::BRICKS,
            Self::Chemicals         => Self::CHEMICALS,
            Self::Clothes           => Self::CLOTHES,
            Self::Concrete          => Self::CONCRETE,
            Self::ElectroComponents => Self::ELECTRO_COMP,
            Self::Electricity       => Self::ELECTRICITY,
            Self::Electronics       => Self::ELECTRONICS,
            Self::Food              => Self::FOOD,
            Self::Gravel            => Self::GRAVEL,
            Self::MechComponents    => Self::MECH_COMP,
            Self::Meat              => Self::MEAT,
            Self::NuclearFuel       => Self::NUCLEAR_FUEL,
            Self::Oil               => Self::OIL,
            Self::Crops             => Self::CROPS,
            Self::PrefabPanels      => Self::PREFABS,
            Self::Steel             => Self::STEEL,
            Self::UF6               => Self::UF_6,
            Self::Uranium           => Self::URANIUM,
            Self::Wood              => Self::WOOD,
            Self::Workers           => Self::WORKERS,
            Self::Yellowcake        => Self::YELLOWCAKE,
        };

        write!(f, "{}", s)
    }
}


impl Display for ParticleType {
    fn fmt(&self, f: &mut Formatter) -> Result<(), Error> {
        let s = match self {
            Self::ResidentialHeating => Self::RESIDENTIAL_HEATING,
            Self::BigBlack    => Self::FACTORY_BIG_BLACK,
            Self::MediumBlack => Self::FACTORY_MEDIUM_BLACK,
            Self::SmallBlack  => Self::FACTORY_SMALL_BLACK,
            Self::BigGray     => Self::FACTORY_BIG_GRAY,
            Self::MediumGray  => Self::FACTORY_MEDIUM_GRAY,
            Self::SmallGray   => Self::FACTORY_SMALL_GRAY,
            Self::BigWhite    => Self::FACTORY_BIG_WHITE,
            Self::MediumWhite => Self::FACTORY_MEDIUM_WHITE,
            Self::SmallWhite  => Self::FACTORY_SMALL_WHITE,
        };

        write!(f, "{}", s)
    }
}


impl Display for Point3f {
    fn fmt(&self, f: &mut Formatter) -> Result<(), Error> {
        write!(f, "({}, {}, {})", self.x, self.y, self.z)
    }
}

impl Display for Rect {
    fn fmt(&self, f: &mut Formatter) -> Result<(), Error> {
        write!(f, "({}, {}, {}, {})", self.x1, self.z1, self.x2, self.z2)
    }
}

impl Display for QuotedStringParam<'_> {
    fn fmt(&self, f: &mut Formatter) -> Result<(), Error> {
        let Self(s) = self;
        write!(f, "\"{}\"", s)
    }
}

impl Display for IdStringParam<'_> {
    fn fmt(&self, f: &mut Formatter) -> Result<(), Error> {
        let Self(s) = self;
        write!(f, "{}", s)
    }
}

impl Display for StrValue<'_> {
    fn fmt(&self, f: &mut Formatter) -> Result<(), Error> {
        let s: &str = match self {
            Self::Borrowed(s) => s,
            Self::Owned(s) => s.as_str()
        };

        write!(f, "{}", s)
    }
}
