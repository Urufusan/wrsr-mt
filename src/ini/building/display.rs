use std::fmt::{Formatter, Error, Display};

use super::{BuildingType,
            StorageCargoType,
            ConstructionPhase,
            ConstructionAutoCost,
            ResourceType,
            ParticleType,
            Token,
            StrValue,
            QuotedStringParam,
            IdStringParam,
           };



impl Display for BuildingType {
    fn fmt(&self, f: &mut Formatter) -> Result<(), Error> {
        let s = match self {
            Self::AirplaneGate           => Self::AIRPLANE_GATE,
            Self::AirplaneParking        => Self::AIRPLANE_PARKING,
            Self::AirplaneTower          => Self::AIRPLANE_TOWER,
            Self::Attraction             => Self::ATTRACTION,
            Self::Broadcast              => Self::BROADCAST,
            Self::CarDealer              => Self::CAR_DEALER,
            Self::CargoStation           => Self::CARGO_STATION,
            Self::Church                 => Self::CHURCH,
            Self::Cityhall               => Self::CITYHALL,
            Self::ConstructionOffice     => Self::CONSTRUCTION_OFFICE,
            Self::ConstructionOfficeRail => Self::CONSTRUCTION_OFFICE_RAIL,
            Self::ContainerFacility      => Self::CONTAINER_FACILITY,
            Self::CoolingTower           => Self::COOLING_TOWER,
            Self::Customhouse            => Self::CUSTOMHOUSE,
            Self::DistributionOffice     => Self::DISTRIBUTION_OFFICE,
            Self::ElectricExport         => Self::ELETRIC_EXPORT,
            Self::ElectricImport         => Self::ELETRIC_IMPORT,
            Self::Engine                 => Self::ENGINE,
            Self::Factory                => Self::FACTORY,
            Self::Farm                   => Self::FARM,
            Self::Field                  => Self::FIELD,
            Self::Firestation            => Self::FIRESTATION,
            Self::ForkliftGarage         => Self::FORKLIFT_GARAGE,
            Self::GasStation             => Self::GAS_STATION,
            Self::HeatingEndstation      => Self::HEATING_ENDSTATION,
            Self::HeatingPlant           => Self::HEATING_PLANT,
            Self::HeatingSwitch          => Self::HEATING_SWITCH,
            Self::Hospital               => Self::HOSPITAL,
            Self::Hotel                  => Self::HOTEL,
            Self::Kindergarten           => Self::KINDERGARTEN,
            Self::Kino                   => Self::KINO,
            Self::Living                 => Self::LIVING,
            Self::MineBauxite            => Self::MINE_BAUXITE,
            Self::MineCoal               => Self::MINE_COAL,
            Self::MineGravel             => Self::MINE_GRAVEL,
            Self::MineIron               => Self::MINE_IRON,
            Self::MineOil                => Self::MINE_OIL,
            Self::MineUranium            => Self::MINE_URANIUM,
            Self::MineWood               => Self::MINE_WOOD,
            Self::Monument               => Self::MONUMENT,
            Self::Parking                => Self::PARKING,
            Self::PassangerStation       => Self::PASSANGER_STATION,
            Self::PollutionMeter         => Self::POLLUTION_METER,
            Self::Powerplant             => Self::POWERPLANT,
            Self::ProductionLine         => Self::PRODUCTION_LINE,
            Self::Pub                    => Self::PUB,
            Self::RailTrafo              => Self::RAIL_TRAFO,
            Self::Raildepo               => Self::RAILDEPO,
            Self::Roaddepo               => Self::ROADDEPO,
            Self::School                 => Self::SCHOOL,
            Self::ShipDock               => Self::SHIP_DOCK,
            Self::Shop                   => Self::SHOP,
            Self::Sport                  => Self::SPORT,
            Self::Storage                => Self::STORAGE,
            Self::Substation             => Self::SUBSTATION,
            Self::Transformator          => Self::TRANSFORMATOR,
            Self::University             => Self::UNIVERSITY,
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

/*
impl<'a, P1, P2, P3, P4, P5, P6> Display for TokenParams6<'a, P1, P2, P3, P4, P5, P6>
where P1: ParamParser<'a>, P1::Output: Display,
      P2: ParamParser<'a>, P2::Output: Display,
      P3: ParamParser<'a>, P3::Output: Display,
      P4: ParamParser<'a>, P4::Output: Display,
      P5: ParamParser<'a>, P5::Output: Display,
      P6: ParamParser<'a>, P6::Output: Display,
{
    fn fmt(&self, f: &mut Formatter) -> Result<(), Error> {
        write!(f, "{{ {} {} {} {} {} {} }}", self.p1, self.p2, self.p3, self.p4, self.p5, self.p6)
    }
}
*/

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

impl<'a> Display for Token<'a> {
    fn fmt(&self, f: &mut Formatter) -> Result<(), Error> {
        write!(f, "$")?;
        match self {
            Self::NameStr(p)                       => write!(f, "{} \"{}\"", Self::NAME_STR, p),
//            Self::Name(p)                      => write!(f, "{}: {}", Self::NAME, p),
            Self::BuildingType(p)                  => write!(f, "TYPE_{}", p),
            Self::CivilBuilding                    => write!(f, "{}", Self::CIVIL_BUILDING),
            Self::QualityOfLiving(x)               => write!(f, "{} {}", Self::QUALITY_OF_LIVING, x),
            Self::Storage((t, x))                  => write!(f, "{} {} {}", Self::STORAGE, t, x),

            Self::ConnectionPedestrian((x, y))     => write!(f, "{} {:?} {:?}", Self::CONNECTION_PEDESTRIAN, x, y),
            Self::ConnectionRoadDead(x)            => write!(f, "{} {:?}", Self::CONNECTION_ROAD_DEAD, x),
            Self::ConnectionsRoadDeadSquare((x,y)) => write!(f, "{} {:?} {:?}", Self::CONNECTIONS_ROAD_DEAD_SQUARE, x, y),

            Self::Particle((t, x, a, s))           => write!(f, "{} {} {:?} {} {}", Self::PARTICLE, t, x, a, s),
            Self::TextCaption((x, y))              => write!(f, "{} {:?} {:?}", Self::TEXT_CAPTION, x, y),

            Self::CostWork((t, x))                 => write!(f, "{} {} {}", Self::COST_WORK, t, x),
            Self::CostWorkBuildingNode(n)          => write!(f, "{} {}", Self::COST_WORK_BUILDING_NODE, n),
            Self::CostResource((t, x))             => write!(f, "{} {} {}", Self::COST_RESOURCE, t, x),
            Self::CostResourceAuto((t, x))         => write!(f, "{} {} {}", Self::COST_RESOURCE_AUTO, t, x),
            Self::CostWorkVehicleStation((x, y))   => write!(f, "{} {:?} {:?}", Self::COST_WORK_VEHICLE_STATION, x, y),
            Self::CostWorkVehicleStationNode(p)    => write!(f, "{} {}", Self::COST_WORK_VEHICLE_STATION_NODE, p),
        }
    }
}
