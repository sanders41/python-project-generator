#!/bin/bash

project_name="My Project"
project_slug=""
source_dir=""
project_description="Test Project"
creator="Arthur Dent"
creator_email="arthur@heartofgold.com"
license="1"
copyright_year="2023"
version=""
python_version=""
min_python_version=""
gha_versions=""
application=""
project_manager="1"

# Check for user provided project manager input
if [ $# -gt 1 ]; then
  if [ $2 -lt 1 ] || [ $2 -gt 5 ]; then
    echo "Invalid project_manager value"
    exit 1
  else
    project_manager=$2
  fi
fi

# Check for user provided application input
if [ $# -gt 0 ]; then
  if [ $1 = "application" ]; then
    application="1"
  elif [ $1 = "lib" ]; then
    application="2"
  else
    echo "Invalid application value"
    exit 1
  fi
fi

max_line_length=""
use_dependabot=""
use_continuous_deployment=""
use_release_drafter=""
use_multi_os_ci=""
pyo3_python_manager=""

if [[ project_manager -eq 3 ]]; then
  ./target/release/python-project create << EOF
$project_name
$project_slug
$source_dir
$project_description
$creator
$creator_email
$license
$copyright_year
$version
$python_version
$min_python_version
$gha_versions
$project_manager
$pyo3_python_manager
$application
$max_line_length
$use_dependabot
$use_continuous_deployment
$use_release_drafter
$use_multi_os_ci
EOF
else
  ./target/release/python-project create << EOF
$project_name
$project_slug
$source_dir
$project_description
$creator
$creator_email
$license
$copyright_year
$version
$python_version
$min_python_version
$gha_versions
$project_manager
$application
$max_line_length
$use_dependabot
$use_continuous_deployment
$use_release_drafter
$use_multi_os_ci
EOF
fi

if [ ! -d $project_slug ]; then
  echo "Directory not created"
  exit 1
fi
