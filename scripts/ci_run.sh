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
max_line_length=""
use_dependabot=""
use_continuous_deployment=""
use_release_drafter=""
use_multi_os_ci=""

./target/release/python-project-generator << EOF
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
$application
$max_line_length
$use_dependabot
$use_continuous_deployment
$use_release_drafter
$use_multi_os_ci
EOF