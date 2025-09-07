// src/simulacion.rs

// Este módulo actúa como el "corazón" o "motor" de la simulación.
// Orquesta las interacciones entre las entidades y gestiona el paso del tiempo.
// Es independiente de la visualización.

use crate::entidades::*;
use rand::thread_rng;

/// Contiene el estado completo de la simulación en un momento dado.
pub struct Simulacion {
    pub dia: u32,
    pub presas: Vec<Box<dyn Presa>>,
    pub depredador: Depredador,
    next_id: u32, // Un contador para asegurar que cada nueva presa tenga un ID único.
}

impl Simulacion {
    /// Crea una nueva instancia de la simulación con las poblaciones iniciales.
    pub fn new() -> Self {
        let mut rng = thread_rng();
        let mut presas: Vec<Box<dyn Presa>> = Vec::new();
        let mut current_id = 0;

        // Poblar el mundo con conejos iniciales.
        for _ in 0..N_CONEJOS_INICIAL {
            presas.push(Box::new(Conejo::new(current_id, &mut rng)));
            current_id += 1;
        }
        // Poblar el mundo con cabras iniciales.
        for _ in 0..N_CABRAS_INICIAL {
            presas.push(Box::new(Cabra::new(current_id, &mut rng)));
            current_id += 1;
        }

        Self {
            dia: 0,
            presas,
            depredador: Depredador::new(DEPREDADOR_RESERVA_INICIAL_KG),
            next_id: current_id,
        }
    }

    /// Avanza la simulación un día, ejecutando todas las fases en orden.
    pub fn avanzar_dia(&mut self) {
        // Si la simulación ya terminó (depredador muerto o presas extintas), no hace nada.
        if !self.depredador.vivo || self.presas.is_empty() {
            return;
        }

        self.dia += 1;
        let mut rng = thread_rng();
        let mut nuevas_crias: Vec<Box<dyn Presa>> = Vec::new();

        // --- FASE 1: DEPREDADOR ---
        // El depredador consume su reserva y, si está vivo, intenta cazar.
        self.depredador.consumir_reserva();
        if self.depredador.vivo {
            self.depredador.cazar(&mut self.presas, &mut rng);
        }

        // --- FASE 2: PRESAS ---
        // Cada presa envejece y tiene la oportunidad de reproducirse.
        for presa in &mut self.presas {
            presa.envejecer();
            nuevas_crias.extend(presa.reproducirse(&mut rng, &mut self.next_id));
        }

        // --- FASE 3: CENSO Y LIMPIEZA ---
        // Se añaden las nuevas crías a la población.
        self.presas.extend(nuevas_crias);
        // Se eliminan de la lista todas las presas que han muerto en este día.
        self.presas.retain(|p| p.esta_viva());
    }
}