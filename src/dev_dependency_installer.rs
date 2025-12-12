use anyhow::{bail, Result};

use crate::{
    package_version::PythonPackage,
    project_generator::{determine_dev_packages, format_package_with_extras},
    project_info::{ProjectInfo, ProjectManager, Pyo3PythonManager},
};

pub fn install_dev_dependencies(project_info: &ProjectInfo) -> Result<()> {
    match project_info.project_manager {
        ProjectManager::Uv => uv_dev_dependency_installer(project_info)?,
        ProjectManager::Poetry => poetry_dev_dependency_installer(project_info)?,
        ProjectManager::Setuptools => setuptools_dev_dependency_installer(project_info)?,
        ProjectManager::Pixi => pixi_dev_dependency_installer(project_info)?,
        ProjectManager::Maturin => maturin_dev_dependency_installer(project_info)?,
    };

    Ok(())
}

fn uv_dev_dependency_installer(project_info: &ProjectInfo) -> Result<()> {
    let packages = determine_dev_packages(project_info)?;
    let package_specs: Vec<String> = packages.iter().map(format_package_with_extras).collect();

    let mut args = vec!["add", "--group=dev"];
    let package_refs: Vec<&str> = package_specs.iter().map(|s| s.as_str()).collect();
    args.extend(package_refs);

    let output = std::process::Command::new("uv")
        .args(args)
        .current_dir(project_info.base_dir())
        .output()?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        bail!("Failed to install dev dependencies: {stderr}");
    }

    Ok(())
}

fn poetry_dev_dependency_installer(project_info: &ProjectInfo) -> Result<()> {
    let packages = determine_dev_packages(project_info)?;

    for package in packages {
        let package_spec = format_package_with_extras(&package);
        let mut args = vec!["add", "--group=dev"];

        if package == PythonPackage::Tomli {
            args.extend(&[&package_spec, "--python", "<3.11"]);
        } else {
            args.push(&package_spec);
        }

        let output = std::process::Command::new("poetry")
            .args(args)
            .current_dir(project_info.base_dir())
            .output()?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            bail!("Failed to install {}: {stderr}", package);
        }
    }

    Ok(())
}

fn setuptools_dev_dependency_installer(project_info: &ProjectInfo) -> Result<()> {
    let venv_path = project_info.base_dir().join(".venv");
    if !venv_path.exists() {
        let venv_output = std::process::Command::new("python")
            .args(["-m", "venv", ".venv"])
            .current_dir(project_info.base_dir())
            .output()?;

        if !venv_output.status.success() {
            let stderr = String::from_utf8_lossy(&venv_output.stderr);
            bail!("Failed to create virtual environment: {stderr}");
        }
    }

    let packages = determine_dev_packages(project_info)?;
    let package_specs: Vec<String> = packages.iter().map(format_package_with_extras).collect();

    let mut args = vec!["-m", "pip", "install"];
    let package_refs: Vec<&str> = package_specs.iter().map(|s| s.as_str()).collect();
    args.extend(package_refs);

    let output = std::process::Command::new(".venv/bin/python")
        .args(args)
        .current_dir(project_info.base_dir())
        .output()?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        bail!("Failed to install dev dependencies: {stderr}");
    }

    let freeze = std::process::Command::new(".venv/bin/python")
        .args(["-m", "pip", "freeze"])
        .current_dir(project_info.base_dir())
        .output()?;

    if !freeze.status.success() {
        let stderr = String::from_utf8_lossy(&freeze.stderr);
        bail!("Failed to get pip freeze output: {stderr}");
    }

    let requirements_path = project_info.base_dir().join("requirements-dev.txt");
    std::fs::write(requirements_path, freeze.stdout)?;

    Ok(())
}

fn pixi_dev_dependency_installer(project_info: &ProjectInfo) -> Result<()> {
    let packages = determine_dev_packages(project_info)?;
    let package_specs: Vec<String> = packages.iter().map(format_package_with_extras).collect();

    let mut args = vec!["add", "--feature", "dev"];
    let package_refs: Vec<&str> = package_specs.iter().map(|s| s.as_str()).collect();
    args.extend(package_refs);

    let output = std::process::Command::new("pixi")
        .args(args)
        .current_dir(project_info.base_dir())
        .output()?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        bail!("Failed to install dev dependencies: {stderr}");
    }

    Ok(())
}

fn maturin_dev_dependency_installer(project_info: &ProjectInfo) -> Result<()> {
    if let Some(pyo3_python_manager) = &project_info.pyo3_python_manager {
        match pyo3_python_manager {
            Pyo3PythonManager::Uv => uv_dev_dependency_installer(project_info),
            Pyo3PythonManager::Setuptools => setuptools_dev_dependency_installer(project_info),
        }
    } else {
        bail!("No Python project manager provided for PyO3 project");
    }
}
