use crate::structs::project::ProjectResponse;

const JWT: &str = "JWT";

pub async fn get_projects() -> Result<ProjectResponse, Box<dyn std::error::Error>> {
    let url = format!(
        "{}/rest/api/3/project/search?maxResults=1000",
        crate::jira::JIRA_URL
    );
    let resp = crate::jira::jira_service::get::<ProjectResponse>(url, &JWT.to_owned()).await?;
    Ok(resp)
}

pub async fn get_project_by_id(
    project_id: String,
) -> Result<ProjectResponse, Box<dyn std::error::Error>> {
    let url = format!(
        "{}/rest/api/3/project/{}",
        crate::jira::JIRA_URL,
        project_id
    );
    let resp = crate::jira::jira_service::get::<ProjectResponse>(url, &JWT.to_owned()).await?;
    Ok(resp)
}

pub async fn post_project(body: String) -> Result<ProjectResponse, Box<dyn std::error::Error>> {
    let url = format!("{}/rest/api/3/project", crate::jira::JIRA_URL);
    let resp =
        crate::jira::jira_service::post::<ProjectResponse>(url, &JWT.to_owned(), body).await?;
    Ok(resp)
}

pub async fn put_project(
    project_id: String,
    body: String,
) -> Result<ProjectResponse, Box<dyn std::error::Error>> {
    let url = format!(
        "{}/rest/api/3/project/{}",
        crate::jira::JIRA_URL,
        project_id
    );
    let resp =
        crate::jira::jira_service::put::<ProjectResponse>(url, &JWT.to_owned(), body).await?;
    Ok(resp)
}

pub async fn delete_project(
    project_id: String,
    body: String,
) -> Result<ProjectResponse, Box<dyn std::error::Error>> {
    let url = format!(
        "{}/rest/api/3/project/{}",
        crate::jira::JIRA_URL,
        project_id
    );
    let resp =
        crate::jira::jira_service::delete::<ProjectResponse>(url, &JWT.to_owned(), body).await?;
    Ok(resp)
}
