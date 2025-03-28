use std::{
    fs::{self, File},
    io::{BufRead, BufReader, BufWriter, Error, ErrorKind, Write},
};

use dirs::config_dir;

use crate::error::OpenRouterError;

/// Stores the user's preferred model names in a file within the user's config directory.
/// This allows the application to remember the user's model preferences across sessions.
///
/// # Arguments
///
/// * `model_names` - A slice of model names to set as preferred.
///
/// # Returns
///
/// * `Result<(), OpenRouterError>` - Ok if the operation was successful, or an error if it failed.
pub fn set_preferred_models(model_names: &[&str]) -> Result<(), OpenRouterError> {
    // Retrieve the user's config directory to store the preferred models
    let config_dir = config_dir()
        .ok_or_else(|| Error::new(ErrorKind::NotFound, "Config directory not found"))?;

    // Ensure the openrouter directory exists within the config directory
    let openrouter_dir = config_dir.join("openrouter");
    if !openrouter_dir.exists() {
        fs::create_dir_all(&openrouter_dir)?;
    }

    // Create or open the selected_models file within the openrouter directory
    let selected_models_file = openrouter_dir.join("selected_models");
    let mut file = BufWriter::new(File::create(selected_models_file)?);

    // Write each model name to the file on a new line
    for model_name in model_names {
        writeln!(file, "{}", model_name)?;
    }
    Ok(())
}

/// Retrieves the user's preferred model names from a file within the user's config directory.
/// This allows the application to load the user's model preferences when it starts.
///
/// # Returns
///
/// * `Result<Vec<String>, OpenRouterError>` - A vector of preferred model names if found, or an error if it failed.
pub fn get_preferred_models() -> Result<Vec<String>, OpenRouterError> {
    // Retrieve the user's config directory to read the preferred models
    let config_dir = config_dir()
        .ok_or_else(|| Error::new(ErrorKind::NotFound, "Config directory not found"))?;

    // Ensure the openrouter directory exists within the config directory
    let openrouter_dir = config_dir.join("openrouter");
    if !openrouter_dir.exists() {
        fs::create_dir_all(&openrouter_dir)?;
    }

    // Open the selected_models file within the openrouter directory
    let selected_models_file = openrouter_dir.join("selected_models");
    let file = BufReader::new(File::open(selected_models_file)?);

    // Read each line from the file and collect them into a vector of model names
    let mut model_names = Vec::new();
    for line in file.lines() {
        let line = line?;
        model_names.push(line);
    }
    Ok(model_names)
}
