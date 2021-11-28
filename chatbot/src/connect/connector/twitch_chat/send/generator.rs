use super::task::SendTask;

pub fn get_login_tasks<'a>(
    password: &'a str,
    user_name: &'a str,
    channel: &'a str,
) -> Vec<SendTask<'a>> {
    return vec![
        SendTask::ProvideLoginPassword(password),
        SendTask::ProvideLoginUserName(user_name),
        SendTask::JoinChannel(channel),
        SendTask::RequestCapabilities("membership"),
        SendTask::RequestCapabilities("tags"),
    ];
}
