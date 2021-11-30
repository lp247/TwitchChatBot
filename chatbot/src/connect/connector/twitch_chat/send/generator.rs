use super::task::SendTask;

pub fn get_login_tasks<'a>(
    password: &'a str,
    user_name: &'a str,
    channel: &'a str,
) -> Vec<SendTask> {
    return vec![
        SendTask::ProvideLoginPassword(password.to_string()),
        SendTask::ProvideLoginUserName(user_name.to_string()),
        SendTask::JoinChannel(channel.to_string()),
        SendTask::RequestCapabilities("membership".to_string()),
        SendTask::RequestCapabilities("tags".to_string()),
    ];
}
