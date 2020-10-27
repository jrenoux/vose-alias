//! This module is an implementation of the Vose-Alias method, to sample an element from a list, given a  discrete probability distribution.
//!
//! This module contains function to create the Probability and Alias tables and sample from them. 
//!
//! The algorithm implemented follows the explanation given on [this page](https://www.keithschwarz.com/darts-dice-coins/)
//!


use std::fmt;
use std::fmt::Display;
use std::hash::Hash;
use std::fmt::Debug;
use float_cmp::*;
use std::collections::HashMap;

use rand::seq::SliceRandom;
use rand::Rng;


/////////////////////////////////////////////
// Structure Definition and Implementation //
/////////////////////////////////////////////
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
#[derive(Debug, Clone)]
pub struct VoseAlias <T> where T: Display + Copy + Hash + Eq + Debug{
    pub elements:Vec<T>,
    pub alias:HashMap<T, T>,
    pub prob:HashMap<T, f32>,
    _private:()
    
}


impl<T> VoseAlias<T>
where T: Display + Copy + Hash + Eq + Debug {

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
    /// let va = VoseAlias::new(vec![1, 2, 3, 4], vec![0.5, 0.2, 0.2, 0.1]);
    /// let element = va.sample();
    /// println!("{}", element);
    /// 
    /// ```
    pub fn sample(&self) -> T {
	let (i, num) = self.roll_die_and_flip_coin();
	return self.select_element(i, num);
    }


    /// This function rolls the die and flip the coin to select the right element using `rand` usual RNG. It returns the generated number. This function is used by the `sample` function and has been decoupled from the `sample` function to allow unit tests on the `sample` function, using pre-determined series of numbers. 
    fn roll_die_and_flip_coin(&self) -> (T, u16) {
	let i:T;
	match self.elements.choose(&mut rand::thread_rng()) {
	    Some(e) => i = *e,
	    None => panic!("Internal error. The element vector is empty. If this happened, please fill in an issue report."),
	}
	let num = rand::thread_rng().gen_range(0, 101);

	return (i, num);
	
    }


    /// This function selects an element from the VoseAlias table given a die (a column) and a coin (the element or its alias). This function has been separated from the `sample` function to allow unit testing, but should never be called by itself. 
    fn select_element(&self, die:T, coin:u16) -> T {
	// choose randomly an element from the element vector
	let p_i:f32;
	match self.prob.get(&die) {
	    Some(p) => p_i = *p,
	    None => panic!("Internal error. The probability vector is empty. If this happened, please fill in an issue report."),
	}
	if (coin as f32) <= (p_i * 100.0) {
	    return die;
	}
	else {
	    match self.alias.get(&die) {
		Some(alias_i) => return *alias_i,
		None => panic!("Internal error. No alias found for element {:?}. If this happened, please fill in an issue report.", die),
	    }
	};
    }
    
}


////////////////////////////
// Traits Implementation  //
////////////////////////////
impl <T> Display for VoseAlias<T>
where T: Display + Copy + Hash + Eq + Debug {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
	// format the elements
	let mut str_elements = String::from("[ ");
	for e in &self.elements {
	    str_elements = str_elements + &e.to_string() + " ";
	}
	str_elements = str_elements + "]";

	// format the alias table
	let mut str_alias = String::from("{ ");
	for k in self.alias.keys() {
	    let a:T;
	    match self.alias.get(&k) {
		Some(element) => a = *element,
		None => panic!("Internal error. The alias map does not contain element for {}. If you encountered this error, please fill in an issue report.", k),
	    }
	    str_alias = str_alias + &String::from(format!("{}:{}, ", k, a));
	}
	// remove the last two characters, that are not needed for the last element
	str_alias = str_alias[..str_alias.len() - 2].to_string() + " }";

	// fomat the probability table
	let mut str_prob = String::from("{");
	for k in self.prob.keys() {
	    let p:f32;
	    match self.prob.get(&k) {
		Some(element) => p = *element,
		None => panic!("Internal error. The alias map does not contain element for {}. If you encountered this error, please fill in an issue report.", k),
	    }
	    str_prob = str_prob + &String::from(format!("{}:{:.2}, ", k, p));
	}
	// remove the last two characters, that are not needed for the last element
	str_prob = str_prob[..str_prob.len() - 2].to_string() + " }";

	// return all of this in a nice string
	write!(f, "{{ elements: {}, alias: {}, prob: {}}}", str_elements, str_alias, str_prob)
    }
}

impl<T> PartialEq for VoseAlias<T>
where T:Display + Copy + Hash + Eq + Debug {
    fn eq(&self, other: &Self) -> bool {
	self.alias == other.alias
    }
    
}


impl <T> Eq for VoseAlias<T>
where T:Display + Copy + Hash + Eq + Debug{
}







///////////
// Tests //
///////////
#[cfg(test)]
mod tests{
    use super::*;

    ////////////////////////////////////////
    // Tests of the Struct Implementation //
    ////////////////////////////////////////
    #[test]
    fn construction_ok() {
        VoseAlias::new(vec![1, 2, 3, 4], vec![0.5, 0.2, 0.2, 0.1]);
    }

    #[test]
    #[should_panic]
    fn size_not_ok() {
        VoseAlias::new(vec![1, 2, 3], vec![0.5, 0.2, 0.2, 0.1]);
    }

    #[test]
    #[should_panic]
    fn sum_not_ok() {
        VoseAlias::new(vec![1, 2, 3, 4], vec![0.5, 0.2, 0.2, 0.]);
    }

    #[test]
    #[should_panic]
    fn new_empty_vectors() {
	let element_vector:Vec<u16> = Vec::new();
	let probability_vector:Vec<f32> = Vec::new();
	VoseAlias::new(element_vector, probability_vector);
    }
    
    #[test]
    fn test_roll_die_flip_coin() {
	let element_vector = vec![1, 2, 3, 4];
	let va = VoseAlias::new(element_vector.clone(), vec![0.5, 0.2, 0.2, 0.1]);
	let (die, coin) = va.roll_die_and_flip_coin();
	assert!(element_vector.contains(&die));
	assert!(coin <= 100);
    }

    #[test]
    fn test_select_element_ok() {
	let va = VoseAlias::new(vec!["orange", "yellow", "green", "turquoise", "grey", "blue", "pink"], vec![0.125, 0.2, 0.1, 0.25, 0.1, 0.1, 0.125]);
	// column orange / alias yellow
	let element = va.select_element("orange", 0);
	assert!(element == "orange");
	let element = va.select_element("orange", 87);
	assert!(element == "orange");
	let element = va.select_element("orange", 88);
	assert!(element == "yellow");
	let element = va.select_element("orange", 100);
	assert!(element == "yellow");

	// column yellow / no alias
	let element = va.select_element("yellow", 0);
	assert!(element == "yellow");
	let element = va.select_element("yellow", 100);
	assert!(element == "yellow");

	// column green / alias turquoise
	let element = va.select_element("green", 0);
	assert!(element == "green");
	let element = va.select_element("green", 70);
	assert!(element == "green");
	let element = va.select_element("green", 71);
	assert!(element == "turquoise");
	let element = va.select_element("green", 100);
	assert!(element == "turquoise");

	// column turquoise / alias yellow
	let element = va.select_element("turquoise", 0);
	assert!(element == "turquoise");
	let element = va.select_element("turquoise", 72);
	assert!(element == "turquoise");
	let element = va.select_element("turquoise", 73);
	assert!(element == "yellow");
	let element = va.select_element("turquoise", 100);
	assert!(element == "yellow");

	// column grey / alias turquoise
	let element = va.select_element("grey", 0);
	assert!(element == "grey");
	let element = va.select_element("grey", 70);
	assert!(element == "grey");
	let element = va.select_element("grey", 71);
	assert!(element == "turquoise");
	let element = va.select_element("grey", 100);
	assert!(element == "turquoise");

	// column blue / alias turquoise
	let element = va.select_element("blue", 0);
	assert!(element == "blue");
	let element = va.select_element("blue", 70);
	assert!(element == "blue");
	let element = va.select_element("blue", 71);
	assert!(element == "turquoise");
	let element = va.select_element("blue", 100);
	assert!(element == "turquoise");

	// column pink / alias turquoise
	let element = va.select_element("pink", 0);
	assert!(element == "pink");
	let element = va.select_element("pink", 87);
	assert!(element == "pink");
	let element = va.select_element("pink", 88);
	assert!(element == "turquoise");
	let element = va.select_element("pink", 100);
	assert!(element == "turquoise");
    }


    #[test]
    #[should_panic]
    fn select_element_proba_too_high() {
	let va = VoseAlias::new(vec!["orange", "yellow", "green", "turquoise", "grey", "blue", "pink"], vec![0.125, 0.2, 0.1, 0.25, 0.1, 0.1, 0.125]);
	va.select_element("yellow", 101);
    }

    #[test]
    #[should_panic]
    fn select_element_not_in_list() {
	let va = VoseAlias::new(vec!["orange", "yellow", "green", "turquoise", "grey", "blue", "pink"], vec![0.125, 0.2, 0.1, 0.25, 0.1, 0.1, 0.125]);
	va.select_element("red", 100);
    }



    ///////////////////////////////////////
    // Tests of the trait implementation //
    ///////////////////////////////////////
    #[test]
    fn test_trait_equal() {
	let va = VoseAlias::new(vec![1, 2, 3, 4], vec![0.5, 0.2, 0.2, 0.1]);
	let va2 = VoseAlias::new(vec![1, 2, 3, 4], vec![0.5, 0.2, 0.2, 0.1]);
	assert!(va==va2);
    }

    #[test]
    fn test_trait_not_equali() {
	let va = VoseAlias::new(vec![1, 2, 3, 4], vec![0.5, 0.2, 0.0, 0.3]);
	let va2 = VoseAlias::new(vec![1, 2, 3, 4], vec![0.5, 0.2, 0.2, 0.1]);
	assert!(va!=va2);
    }
    
}
