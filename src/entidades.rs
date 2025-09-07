// src/entidades.rs

// Este módulo define todas las entidades de la simulación y sus reglas.
// Contiene las "clases base" (traits), las implementaciones concretas (structs),
// y los parámetros que gobiernan el ecosistema.

use rand::{Rng, seq::SliceRandom};
use rand::rngs::ThreadRng; // Se importa el tipo concreto de generador de números aleatorios.

// =================================================
// PARÁMETROS GLOBALES DE LA SIMULACIÓN
// Estas constantes actúan como "perillas" para ajustar el comportamiento del ecosistema.
// =================================================

// --- Población Inicial (AJUSTADO) ---
pub const N_CONEJOS_INICIAL: u32 = 60; 
pub const N_CABRAS_INICIAL: u32 = 25;

// --- Parámetros del Depredador ---
pub const DEPREDADOR_RESERVA_INICIAL_KG: f64 = 900.0; 
pub const DEPREDADOR_CONSUMO_MINIMO_DIARIO_KG: f64 = 3.0;
pub const DEPREDADOR_CONSUMO_OPTIMO_DIARIO_KG: f64 = 5.0;

// --- Parámetros de CONEJO (AJUSTADO) ---
const CONEJO_EDAD_MAXIMA_DIAS: u32 = 1825;
const CONEJO_EDAD_REPRODUCTIVA_DIAS: u32 = 100;
const CONEJO_EDAD_SACRIFICIO_DIAS: u32 = 150;  
const CONEJO_TASA_REPRODUCCION_DIARIA: f64 = 0.05;
const CONEJO_CRIAS_POR_PARTO: (u32, u32) = (3, 6);

// --- Parámetros de CABRA (AJUSTADO) ---
const CABRA_EDAD_MAXIMA_DIAS: u32 = 5475;
const CABRA_EDAD_REPRODUCTIVA_DIAS: u32 = 300;
const CABRA_EDAD_SACRIFICIO_DIAS: u32 = 250;  
const CABRA_TASA_REPRODUCCION_DIARIA: f64 = 0.01;
const CABRA_CRIAS_POR_PARTO: (u32, u32) = (1, 2);

// --- Probabilidades Comunes ---
const PROBABILIDAD_ENFERMAR: f64 = 0.001;
const PROBABILIDAD_NACER_MACHO: f64 = 0.5;

// =================================================
// DEFINICIONES DE TIPOS (ENUMS, STRUCTS, TRAITS)
// =================================================

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Sexo { Macho, Hembra }

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Especie { Conejo, Cabra }

/// El trait `Presa` define un "contrato" de comportamiento común para todas las presas.
/// Esto permite el polimorfismo dinámico (tratar a Conejos y Cabras de la misma manera).
pub trait Presa {
    // Métodos para acceder a los datos internos de forma segura.
    fn id(&self) -> u32;
    fn especie(&self) -> Especie;
    fn sexo(&self) -> Sexo;
    fn edad(&self) -> u32;
    fn peso(&self) -> f64;
    fn esta_viva(&self) -> bool;

    // Métodos que modifican el estado de la presa.
    fn envejecer(&mut self);
    fn reproducirse(&self, rng: &mut ThreadRng, next_id: &mut u32) -> Vec<Box<dyn Presa>>;
}

/// Función de orden superior (concepto funcional) que actúa como una "fábrica".
/// Crea y devuelve una clausura especializada para calcular el peso según la curva de Gompertz.
fn crear_funcion_gompertz(peso_max: f64, tasa_crecimiento: f64, punto_inflexion: f64) -> Box<dyn Fn(u32) -> f64> {
    Box::new(move |edad_dias| {
        let t = edad_dias as f64;
        let exponente_interno = -tasa_crecimiento * (t - punto_inflexion);
        let exponente_externo = -f64::exp(exponente_interno);
        peso_max * f64::exp(exponente_externo)
    })
}

// --- Implementación de CONEJO ---

/// Representa a un conejo individual en la simulación.
pub struct Conejo {
    id: u32,
    edad_dias: u32,
    peso_kg: f64,
    sexo: Sexo,
    vivo: bool,
    crecimiento: Box<dyn Fn(u32) -> f64>,
}

impl Conejo {
    /// Constructor para crear un nuevo Conejo.
    pub fn new(id: u32, rng: &mut ThreadRng) -> Self {
        let sexo = if rng.gen_bool(PROBABILIDAD_NACER_MACHO) { Sexo::Macho } else { Sexo::Hembra };
        let crecimiento = crear_funcion_gompertz(5.0, 0.05, 90.0);
        let peso_inicial = crecimiento(0);
        Self { id, edad_dias: 0, peso_kg: peso_inicial, sexo, vivo: true, crecimiento }
    }
}

/// Implementación del "contrato" `Presa` para la struct `Conejo`.
impl Presa for Conejo {
    fn id(&self) -> u32 { self.id }
    fn especie(&self) -> Especie { Especie::Conejo }
    fn sexo(&self) -> Sexo { self.sexo }
    fn edad(&self) -> u32 { self.edad_dias }
    fn peso(&self) -> f64 { self.peso_kg }
    fn esta_viva(&self) -> bool { self.vivo }

    /// Incrementa la edad, actualiza el peso y gestiona la muerte por vejez o enfermedad.
    fn envejecer(&mut self) {
        self.edad_dias += 1;
        self.peso_kg = (self.crecimiento)(self.edad_dias);
        if self.edad_dias > CONEJO_EDAD_MAXIMA_DIAS || rand::random::<f64>() < PROBABILIDAD_ENFERMAR {
            self.vivo = false;
        }
    }

    /// Gestiona la reproducción si se cumplen las condiciones de edad, sexo y probabilidad.
    fn reproducirse(&self, rng: &mut ThreadRng, next_id: &mut u32) -> Vec<Box<dyn Presa>> {
        let mut crias: Vec<Box<dyn Presa>> = Vec::new();
        if self.sexo == Sexo::Hembra && self.edad_dias >= CONEJO_EDAD_REPRODUCTIVA_DIAS && rng.gen_bool(CONEJO_TASA_REPRODUCCION_DIARIA) {
            let cantidad = rng.gen_range(CONEJO_CRIAS_POR_PARTO.0..=CONEJO_CRIAS_POR_PARTO.1);
            for _ in 0..cantidad {
                crias.push(Box::new(Conejo::new(*next_id, rng)));
                *next_id += 1;
            }
        }
        crias
    }
}

// --- Implementación de CABRA ---

/// Representa a una cabra individual en la simulación.
pub struct Cabra {
    id: u32,
    edad_dias: u32,
    peso_kg: f64,
    sexo: Sexo,
    vivo: bool,
    crecimiento: Box<dyn Fn(u32) -> f64>,
}

impl Cabra {
    /// Constructor para crear una nueva Cabra.
    pub fn new(id: u32, rng: &mut ThreadRng) -> Self {
        let sexo = if rng.gen_bool(PROBABILIDAD_NACER_MACHO) { Sexo::Macho } else { Sexo::Hembra };
        let crecimiento = crear_funcion_gompertz(75.0, 0.01, 180.0);
        let peso_inicial = crecimiento(0);
        Self { id, edad_dias: 0, peso_kg: peso_inicial, sexo, vivo: true, crecimiento }
    }
}

/// Implementación del "contrato" `Presa` para la struct `Cabra`.
impl Presa for Cabra {
    fn id(&self) -> u32 { self.id }
    fn especie(&self) -> Especie { Especie::Cabra }
    fn sexo(&self) -> Sexo { self.sexo }
    fn edad(&self) -> u32 { self.edad_dias }
    fn peso(&self) -> f64 { self.peso_kg }
    fn esta_viva(&self) -> bool { self.vivo }

    fn envejecer(&mut self) {
        self.edad_dias += 1;
        self.peso_kg = (self.crecimiento)(self.edad_dias);
        if self.edad_dias > CABRA_EDAD_MAXIMA_DIAS || rand::random::<f64>() < PROBABILIDAD_ENFERMAR {
            self.vivo = false;
        }
    }

    fn reproducirse(&self, rng: &mut ThreadRng, next_id: &mut u32) -> Vec<Box<dyn Presa>> {
        let mut crias: Vec<Box<dyn Presa>> = Vec::new();
        if self.sexo == Sexo::Hembra && self.edad_dias >= CABRA_EDAD_REPRODUCTIVA_DIAS && rng.gen_bool(CABRA_TASA_REPRODUCCION_DIARIA) {
            let cantidad = rng.gen_range(CABRA_CRIAS_POR_PARTO.0..=CABRA_CRIAS_POR_PARTO.1);
            for _ in 0..cantidad {
                crias.push(Box::new(Cabra::new(*next_id, rng)));
                *next_id += 1;
            }
        }
        crias
    }
}


// --- Implementación del DEPREDADOR ---

/// Representa al único depredador de la simulación.
pub struct Depredador {
    pub reserva_comida_kg: f64,
    pub vivo: bool,
}

impl Depredador {
    pub fn new(reserva_inicial: f64) -> Self {
        Self { reserva_comida_kg: reserva_inicial, vivo: true }
    }

    /// Consume comida de la reserva para sobrevivir, gestionando la muerte por inanición.
    pub fn consumir_reserva(&mut self) {
        if self.reserva_comida_kg >= DEPREDADOR_CONSUMO_OPTIMO_DIARIO_KG {
            self.reserva_comida_kg -= DEPREDADOR_CONSUMO_OPTIMO_DIARIO_KG;
        } else if self.reserva_comida_kg >= DEPREDADOR_CONSUMO_MINIMO_DIARIO_KG {
            self.reserva_comida_kg -= DEPREDADOR_CONSUMO_MINIMO_DIARIO_KG;
        } else {
            // Si no puede consumir ni el mínimo, muere.
            self.vivo = false;
        }
    }

    /// Implementa la lógica de caza siguiendo las reglas especificadas.
    pub fn cazar(&mut self, presas: &mut Vec<Box<dyn Presa>>, rng: &mut ThreadRng) {
        // 1. Filtrar solo presas que han alcanzado la edad de sacrificio.
        let presas_cazables: Vec<(usize, &Box<dyn Presa>)> = presas.iter().enumerate()
            .filter(|(_, p)| {
                let edad_sacrificio = match p.especie() {
                    Especie::Conejo => CONEJO_EDAD_SACRIFICIO_DIAS,
                    Especie::Cabra => CABRA_EDAD_SACRIFICIO_DIAS,
                };
                p.edad() >= edad_sacrificio && p.esta_viva()
            })
            .collect();

        if presas_cazables.is_empty() { return; } // Si no hay presas válidas, no caza.

        // 2. Encontrar el peso máximo entre las presas cazables.
        let peso_maximo = presas_cazables.iter()
            .map(|(_, p)| p.peso())
            .fold(0.0, f64::max);

        // 3. Obtener los índices de todas las presas que empatan en el peso máximo.
        let mejores_presas_indices: Vec<usize> = presas_cazables.into_iter()
            .filter(|(_, p)| p.peso() >= peso_maximo - 0.01) // Tolerancia para flotantes
            .map(|(i, _)| i)
            .collect();

        // 4. Elegir una al azar de los mejores, removerla y añadir su peso a la reserva.
        if let Some(&indice_a_cazar) = mejores_presas_indices.choose(rng) {
            let presa_cazada = presas.remove(indice_a_cazar);
            self.reserva_comida_kg += presa_cazada.peso();
        }
    }
}