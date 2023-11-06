use phf::phf_map;

pub const MAZES: phf::Map<&'static str, &'static str> = phf_map! {

"open" =>
"███████████████████████████████
█                             █
█                             █
█                             █
█                             █
█                             █
█                             █
█                             █
█                             █
█                             █
█                             █
█                             █
█                             █
█                             █
█                       X     █
█                             █
█                             █
█                             █
█                             █
█                             █
███████████████████████████████",

"basic" =>
"█████████████████████
█   █     █   █   █ █
█ ███ █████ ███ █ █ █
█ █ █ █   █ █     █ █
█ █ █ ███ █ █ ███ ███
█ █ █   █ █ █ █     █
█ █ ███ █ █ █ ███ ███
█           █ █     █
█ █ █ █████ ███████ █
█ █ █     █   █     █
█ █████ █ ███ █ █ ███
█ █ █   █ █   █ █   █
█ █ ███████ █████ █ █
█ █     █ █       █ █
█ █████ █ █ ███████ █
█ █   █   █     █X  █
█ █ ███ ███ ███ █████
█     █ █ █ █ █ █   █
█ █ ███ █ ███ █ █ █ █
█ █         █     █ █
█████████████████████",

"large" =>
"█████████████████████████████████████████████████████████████
█     █ █           █   █         █ █     █     █ █   █     █
█ █ █ █ █████ █ █████████ ███ ███ ███████ █ ███ ███ ███ ███ █
█ █ █   █           █ █   █   █ █       █   █         █ █ █ █
█ ███ ███ ███ █ ███ █ ███ █ █ █ █ █ █████████ █ █████ █ █ █ █
█ █     █   █ █ █ █ █     █ █   █ █ █   █   █ █ █ █ █       █
█████ █████ █████ █ █████ █ █ █████ █ █ █ █ █ ███ █ █████ █ █
█       █         █     █   █ █     █ █ █ █     █ █       █ █
█ █████ ███ ███ █████████ █ █ █ ███ █ █ █ ███ ███ ███████ ███
█ █ █       █ █ █ █ █     █ █ █ █ █ █ █   █ █     █       █ █
███ █ █ ███ █ ███ █ ███ █████ ███ █ ███████ █ █ ███████ ███ █
█     █ █   █       █ █   █       █ █ █   █ █ █       █ █ █ █
█ █████████ █ ███ ███ █████ █ █ ███ █ █ █ █ █ █████ █████ █ █
█   █   █ █ █ █   █ █ █   █ █ █     █ █ █   █ █     █ █ █   █
█ █████ █ █ █ █████ █ █ █████ ███ ███ █████ █ █ █████ █ ███ █
█ █ █   █   █     █   █     █   █   █ █ █     █ █ █         █
█ █ █ █████ █ ███ █ █ ███ ███ █ █████ █ ███ █ ███ █████ ███ █
█     █         █ █ █     █   █ █   █ █ █   █   █ █     █   █
███ █ █████ █████ █ ███ █ █████ █ █ █ █ █████ ███ █ ███ █ ███
█   █     █ █ █   █   █ █ █ █   █ █ █   █     █   █   █ █ █ █
█ ███ █ █ ███ █████ ███ █ █ ███ █ █████ █████ ███ █ ███████ █
█  X█ █ █ █     █     █ █ █ █ █   █       █   █ █ █         █
█████ █ █ █ █ ███ █████████ █ █ ███ █████ █ ███ █ █ █ ███████
█ █   █ █   █ █         █ █   █   █   █   █   █     █     █ █
█ █ ███ █████ █ █████████ ███ ███ █ █ ███ ███ █ █ █████████ █
█ █ █     █ █ █   █     █       █   █ █         █ █ █       █
█ █ ███ █ █ █ █ █████ ███ █████ █ ███████████ █ ███ █████ ███
█     █ █   █   █         █ █   █ █   █   █   █       █ █   █
███ ███████ ███████████ ███ █████ █ █████ █ ███████████ ███ █
█       █ █ █ █   █                   █         █ █       █ █
█ █████ █ ███ █ ███████ ███ ███████████ ███ █████ █ █ █ ███ █
█ █       █     █       █ █     █   █   █ █ █       █ █     █
███ ███ █ ███ ███ █ █ ███ █ ███ █ ███ █ █ █████ █████████ █ █
█ █ █   █       █ █ █ █     █     █ █ █ █ █ █       █   █ █ █
█ ███ █ █ ███████ ███ █ █ █ █████ █ █ █ █ █ █████ ███ ███ ███
█     █ █   █ █   █   █ █ █   █   █ █ █ █       █ █     █   █
█ █ ███ █████ ███ ███████ █████████ █ █ █ █ █ █ █ █ ███████ █
█ █ █     █   █ █ █ █     █ █       █ █ █ █ █ █ █ █   █ █ █ █
█ ███████████ █ █ █ █ █ █ █ ███ ███ █ █ █ ███ ███ █ ███ █ █ █
█ █               █   █ █         █ █ █     █             █ █
█████████████████████████████████████████████████████████████"

};