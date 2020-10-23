//! This module is an implementation of the Vose-Alias method, to sample an element from a list, given a  discrete probability distribution.
//!
//! This module contains function to create the Probability and Alias tables and sample from them. 
//!
//! The algorithm implemented follows the explanation given on [this page](https://www.keithschwarz.com/darts-dice-coins/)
//!



use std::hash::Hash;
use std::fmt::Debug;
use float_cmp::*;
use std::collections::HashMap;

use rand::seq::SliceRandom;
use rand::Rng;



/// A structure containing the necessary Vose-Alias tables. 
///
/// The structure contains the following attributes:
/// 1. A vector containing the elements to sample frmo
/// 2. The Alias table, created from the Vose-Alias initialization step
/// 3. The Probability table, created frmo the Vose-Alias initialization step
///
/// The structure is created by the function `vose_alias::new()`. See its documentation for more details.
///
/// Internally, the elements are used as indexes in `HashMap` and `Vec`. Therefore, the type `T` must implement the following traits:
/// - Copy
/// - Hash
/// - Eq
/// - Debug
#[derive(Debug)]
pub struct VoseAlias <T>{
    pub elements:Vec<T>,
    pub alias:HashMap<T, T>,
    pub prob:HashMap<T, f32>,
    _private:()
    
}


impl<T: Copy + Hash + Eq + Debug> VoseAlias<T> {

    /// Returns the Vose-Alias object containing the element vector as well as the alias and probability tables.
    ///
    /// The `element_vector` contains the list of elements that should be sampled from.
    /// The `probability_vector` contains the discrete probability distribution to be sampled with.
    /// `element_vector` and `probability_vector` should have the same size and `probability_vector` should describe a well-formed probability distribution.
    ///
    /// # Panics
    ///
    /// The function panics in two casese:
    /// 1. the `element_vector` and the `probability_vector` do not contain the same number of elements
    /// 2. the sum of the elements in `probability_vector` is not equal to 1 (with a floating number precision of 0.0001), meaning that `probability_vector` does not describe a well formed probability distribution
    ///
    /// # Examples
    /// ```
    /// use vose_alias::VoseAlias;
    /// 
    /// // Creates a Vose-Alias object from a list of Integer elements
    /// let va = VoseAlias::new(vec![1, 2, 3, 4], vec![0.5, 0.2, 0.2, 0.1]);
    /// ```
    
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
	    // removing the element from small and large
            if let (Some(l), Some(g)) = (small.pop(), large.pop()) {
		// put g in the alias vector
		alias.insert(l, g);
		// getting the probability of the small element
		if let Some(p_l) = scaled_probability_vector.get(&l) {
		    // put it in the prob vector
		    prob.insert(l, *p_l);

		    // update the probability for g
		    if let Some(p_g) = scaled_probability_vector.get(&g) { 
			let new_p_g = (*p_g + *p_l) - 1.0;
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

        VoseAlias {
	    elements: element_vector,
            alias: alias,
            prob: prob,
            _private: ()
        }
    }


    
    /// Returns a sampled element from a previously created Vose-Alias object.
    ///
    /// This function uses a `VoseAlias` object previously created using the method `vose_alias::new()` to sample in linear time an element of type `T`.
    ///
    /// # Panics
    /// This function panics only if the lists created in `vose_alias::new()` are not correctly form, which would indicate a internal bug in the code.
    /// If your code panics while using this function, please fill in an issue report.
    ///
    /// # Examples
    /// ```
    /// use vose_alias::VoseAlias;
    ///
    /// // Samples an integer from a list and prints it. 
    /// let va = VoseAlias::new(vec![1, 2, 3, 4];, vec![0.5, 0.2, 0.2, 0.1]);
    /// let element = va.sample();
    /// println!("{}", element);
    /// 
    /// ```
    pub fn sample(&self) -> T {
	// choose randomly an element from the element vector
	let i:T;
	let p_i:f32;
	match self.elements.choose(&mut rand::thread_rng()) {
	    Some(e) => i = *e,
	    None => panic!("Internal error. The element vector is empty. If this happened, please fill in an issue report."),
	}

	match self.prob.get(&i) {
	    Some(p) => p_i = *p,
	    None => panic!("Internal error. The probability vector is empty. If this happened, please fill in an issue report."),
	}
	let num = rand::thread_rng().gen_range(0, 101);
	if (num as f32) < (p_i * 100.0) {
	    return i;
	}
	else {
	    match self.alias.get(&i) {
		Some(alias_i) => return *alias_i,
		None => panic!("Internal error. No alias found for element {:?}. If this happened, please fill in an issue report.", i),
	    }
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

    #[test]
    fn test_sample() {
	let element_vector = vec![1, 2, 3, 4];
	let va = VoseAlias::new(element_vector.clone(), vec![0.5, 0.2, 0.2, 0.1]);
	let element = va.sample();
	assert!(element_vector.contains(&element));
    }
}
