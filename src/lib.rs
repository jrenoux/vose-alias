use std::hash::Hash;
use std::fmt::Debug;
use std::fmt::Display;
use float_cmp::*;
use std::collections::HashMap;

use rand::seq::SliceRandom;
use rand::Rng;

#[derive(Debug)]
pub struct VoseAlias <T>{
    pub elements:Vec<T>,
    pub alias:HashMap<T, T>,
    pub prob:HashMap<T, f32>,
    _private:()
    
}

impl<T: Display + Copy + Hash + PartialEq + Eq + Debug> VoseAlias<T> {

    pub fn new(element_vector:Vec<T>, probability_vector:Vec<f32>) -> VoseAlias<T> {
        let size_p = probability_vector.len();
        let size_e = element_vector.len();
        // some sanity checks
        if size_p != size_e {
            panic!("Both vectors should contain the same number of elements");
        }

        let mut sum = 0.0;
        for p in &probability_vector {
            sum = sum + p;
        }

        if !approx_eq!(f32, sum, 1.0, ulps=4) {
            panic!("Probability vector does not sum to 1");
        }

        
        // starting the actual init
        let size = probability_vector.len();
        let mut small:Vec<T> = Vec::new();
        let mut large:Vec<T> = Vec::new();
	let mut scaled_probability_vector:HashMap<T, f32> = HashMap::new();

        let mut alias:HashMap<T, T> = HashMap::new();
        let mut prob:HashMap<T, f32> = HashMap::new();

        // multiply each proba by size
        for i in 0..size {
            let p:f32 = probability_vector[i];
            let e:T = element_vector[i];
            let scaled_proba = p * (size as f32);
            scaled_probability_vector.insert(e, scaled_proba);

            if scaled_proba < 1.0 {
                small.push(e);
            }
            else {
                large.push(e);
            }
        }


	// emptying one column first
        while !(small.is_empty() || large.is_empty()) {
	    // DEBUG - print small and large
	    // println!("-----------------------------------------------");
	    // println!("{:?}, {:?}", !small.is_empty(), !large.is_empty());
            // println!("Small");
            // for s in &small {
	    // 	println!("{}", s);
            // }

            // println!("Large");
            // for l in &large {
	    // 	println!("{}", l);
            // }

	    
	    
	    // removing the element from small and large
            if let (Some(l), Some(g)) = (small.pop(), large.pop()) {
		// DEBUG -
		// println!("l = {}, g = {}", l, g);
		
		// put g in the alias vector
		alias.insert(l, g);
		// getting the probability of the small element
		if let Some(p_l) = scaled_probability_vector.get(&l) {
		    // DEBUG
		    // println!("p_l = {}", p_l);
		    
		    // put it in the prob vector
		    prob.insert(l, *p_l);

		    // update the probability for g
		    if let Some(p_g) = scaled_probability_vector.get(&g) { 
			let new_p_g = (*p_g + *p_l) - 1.0;
			// DEBUG -
			// println!("p_g = {}, new_p_g = {}", *p_g, new_p_g);
			
			// update scaled_probability_vector
			scaled_probability_vector.insert(g, new_p_g);
			if new_p_g < 1.0 {
			    small.push(g);
			}
			else {
			    large.push(g);
			}
		    };
		    
		}
	    }
        }

	// finishing the init
	while !large.is_empty() {
	    if let Some(g) = large.pop() {
		// println!("Last but not least: g = {}", g);
		prob.insert(g, 1.0);
	    };
	}

	while !small.is_empty() {
	    if let Some(l) = small.pop() {
		// println!("Last but not least: l = {}", l);
		prob.insert(l, 1.0);
	    }
	}


	// println!("-----------------------------------------------");
	// println!("At the end of the init... Probability vector: ");
	// for key in prob.keys() {
	//     if let Some(value) = prob.get(key) {
	// 	println!("Prob({}) = {}", key, *value);
	// 	if let Some(alias_value) = alias.get(key) {
	// 	    println!("Alias({}) = {}", key, alias_value);
	// 	}
	//     }
	// }

	// println!("{:?}", prob);

	        

        VoseAlias {
	    elements: element_vector,
            alias: alias,
            prob: prob,
            _private: ()
        }
    }

    pub fn sample(&self) -> Option<T> {
	// choose randomly an element from the element vector
	let i = self.elements.choose(&mut rand::thread_rng())?;
	let p_i = self.prob.get(&i)?;
	let num = rand::thread_rng().gen_range(0, 101);
	if (num as f32) < (*p_i * 100.0) {
	    return Some(*i);
	}
	else {
	    let alias_i = self.alias.get(i)?;
	    return Some(*alias_i);
	};
    }

}

#[cfg(test)]
mod tests{
    use super::*;
    
    #[test]
    fn size_ok() {
        VoseAlias::new(vec![1, 2, 3, 4], vec![0.5, 0.2, 0.2, 0.1]);
    }

    #[test]
    #[should_panic]
    fn size_not_ok() {
        VoseAlias::new(vec![1, 2, 3], vec![0.5, 0.2, 0.2, 0.1]);
    }

    #[test]
    fn sum_ok() {
        VoseAlias::new(vec![1, 2, 3, 4], vec![0.5, 0.2, 0.2, 0.1]);
    }

    #[test]
    #[should_panic]
    fn sum_not_ok() {
        VoseAlias::new(vec![1, 2, 3, 4], vec![0.5, 0.2, 0.2, 0.]);
    }
}
