
use crate::ini;

pub fn render(f: &mut ini::RenderIni<'_>, factor: f64) {
    use crate::ini::RenderToken as T;

    for (_, t_state) in f.tokens.iter_mut() {
        t_state.modify(|t| match t {
           T::Light((pt, x))            => Some(T::Light((pt.scaled(factor), *x))),
           T::LightRgb((pt, x, c))      => Some(T::LightRgb((pt.scaled(factor), *x, *c))),
           T::LightRgbBlink((pt, x, c)) => Some(T::LightRgbBlink((pt.scaled(factor), *x, *c))),
            _ => None 
        });
    }
}


pub fn building(f: &mut ini::BuildingIni<'_>, factor: f64) {
    use crate::ini::BuildingToken as T;

    for (_, t_state) in f.tokens.iter_mut() {
        t_state.modify(|t| match t {
            T::VehicleStation((p1, p2))             => Some(T::VehicleStation((p1.scaled(factor), p2.scaled(factor)))),
            T::VehicleStationDetourPoint(p1)        => Some(T::VehicleStationDetourPoint(p1.scaled(factor))),
            T::VehicleStationDetourPid((i, p1))     => Some(T::VehicleStationDetourPid((*i, p1.scaled(factor)))),
            T::VehicleParking((p1, p2))             => Some(T::VehicleParking((p1.scaled(factor), p2.scaled(factor)))),

            T::Connection2Points((t, p1, p2))       => Some(T::Connection2Points((*t, p1.scaled(factor), p2.scaled(factor)))),
            _ => None 
        });
    }
}
