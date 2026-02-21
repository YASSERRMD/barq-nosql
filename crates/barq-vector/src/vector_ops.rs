use rand::Rng;

pub struct VectorOps;

impl VectorOps {
    pub fn normalize(vec: &mut [f32]) {
        let norm = vec.iter().map(|x| x * x).sum::<f32>().sqrt();
        if norm > 0.0 {
            for v in vec.iter_mut() {
                *v /= norm;
            }
        }
    }

    pub fn random(dim: usize) -> Vec<f32> {
        let mut rng = rand::thread_rng();
        (0..dim).map(|_| rng.gen::<f32>()).collect()
    }

    pub fn random_unit(dim: usize) -> Vec<f32> {
        let mut vec = Self::random(dim);
        Self::normalize(&mut vec);
        vec
    }

    pub fn add(a: &[f32], b: &[f32]) -> Vec<f32> {
        a.iter().zip(b.iter()).map(|(x, y)| x + y).collect()
    }

    pub fn scale(vec: &[f32], scalar: f32) -> Vec<f32> {
        vec.iter().map(|x| x * scalar).collect()
    }

    pub fn mean(vectors: &[Vec<f32>]) -> Option<Vec<f32>> {
        if vectors.is_empty() {
            return None;
        }

        let dim = vectors[0].len();
        let mut sum = vec![0.0; dim];

        for vec in vectors {
            if vec.len() != dim {
                return None;
            }
            for (i, v) in vec.iter().enumerate() {
                sum[i] += v;
            }
        }

        let len = vectors.len() as f32;
        for s in sum.iter_mut() {
            *s /= len;
        }

        Some(sum)
    }
}
