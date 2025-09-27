#!/bin/bash

project_name="My Project"
project_slug=""
source_dir=""
project_description="Test Project"
creator="Arthur Dent"
creator_email="arthur@heartofgold.com"
license="1"
fastapi_project="1"
copyright_year=""
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
  if [ $1 = "fastapi" ] ; then
    fastapi_project="1"
    application="1"
  elif [ $1 = "application" ]; then
    fastapi_project="2"
    application="1"
  elif [ $1 = "lib" ]; then
    fastapi_project="2"
    application="2"
  else
    echo "Invalid application value"
    exit 1
  fi
fi

# database_manager=""
max_line_length=""
use_dependabot=""
use_continuous_deployment=""
use_release_drafter=""
pyo3_python_manager=""

if [[ project_manager -eq 3 ]]; then
  ./target/release/python-project create -s << EOF
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
EOF
else
  if [ "$fastapi_project" = "1" ]; then
    ./target/release/python-project create -s << EOF
$project_name
$project_slug
$source_dir
$project_description
$creator
$creator_email
$license
$copyright_year
$version
$fastapi_project
$python_version
$min_python_version
$gha_versions
$project_manager
$max_line_length
$use_dependabot
$use_continuous_deployment
$use_release_drafter
EOF
  else
    ./target/release/python-project create -s << EOF
$project_name
$project_slug
$source_dir
$project_description
$creator
$creator_email
$license
$copyright_year
$version
$fastapi_project
$python_version
$min_python_version
$gha_versions
$project_manager
$application
$max_line_length
$use_dependabot
$use_continuous_deployment
$use_release_drafter
EOF
  fi
fi

if [ ! -d $project_slug ]; then
  echo "Directory not created"
  exit 1
fi
