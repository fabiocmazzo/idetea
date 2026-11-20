//! A escolha do usuário numa `session/request_permission`.
//!
//! Fechar, cancelar ou dispensar o diálogo nunca vira permissão. Não há
//! aprovação automática neste marco.

use agent_client_protocol::v1::schema::{
    PermissionOption, PermissionOptionId, PermissionOptionKind, RequestPermissionOutcome, SelectedPermissionOutcome,
};

/// O que a pessoa fez com o pedido. `Dismiss` é Esc, clique fora ou fechar:
/// a ação não foi escolhida.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UserChoice {
    /// A pessoa escolheu uma opção pelo id que o agente mandou.
    Option(u32),
    /// Permitir uma vez, se o agente ofereceu `allow_once` (senão `allow_always`).
    Allow,
    /// Recusar uma vez, se o agente ofereceu `reject_once` (senão `reject_always`).
    Reject,
    /// Fechar, cancelar ou dispensar. Nunca é aprovação.
    Dismiss,
}

/// Para onde a escolha vai no protocolo. `Dismiss` não responde: o pedido
/// continua aberto. `Cancel` só vale depois de um `session/cancel`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PermissionAnswer {
    /// Ainda esperando. Não manda nada ao agente.
    Pending,
    /// `outcome: selected` com o id da opção.
    Selected(RequestPermissionOutcome),
    /// `outcome: cancelled`, e quem chama manda `session/cancel` antes.
    Cancel,
}

/// Uma opção como a interface mostra, sem os tipos do protocolo.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ShownOption {
    pub id: String,
    pub name: String,
    pub kind: Kind,
}

/// O tipo da opção, para o botão (permitir, recusar).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Kind {
    AllowOnce,
    AllowAlways,
    RejectOnce,
    RejectAlways,
    Other,
}

impl Kind {
    pub fn is_allow(self) -> bool {
        matches!(self, Kind::AllowOnce | Kind::AllowAlways)
    }

    pub fn is_reject(self) -> bool {
        matches!(self, Kind::RejectOnce | Kind::RejectAlways)
    }
}

pub fn shown_options(options: &[PermissionOption]) -> Vec<ShownOption> {
    options
        .iter()
        .map(|o| ShownOption {
            id: option_id(&o.option_id),
            name: o.name.clone(),
            kind: match o.kind {
                PermissionOptionKind::AllowOnce => Kind::AllowOnce,
                PermissionOptionKind::AllowAlways => Kind::AllowAlways,
                PermissionOptionKind::RejectOnce => Kind::RejectOnce,
                PermissionOptionKind::RejectAlways => Kind::RejectAlways,
                _ => Kind::Other,
            },
        })
        .collect()
}

fn option_id(id: &PermissionOptionId) -> String {
    // O id é um newtype em volta de string na maioria das versões; o Display
    // e o Debug cobrem o resto. Preferimos o que vai no fio.
    serde_json::to_value(id).ok().and_then(|v| v.as_str().map(str::to_string)).unwrap_or_else(|| id.to_string())
}

/// Traduz a escolha da pessoa. `Dismiss` devolve [`PermissionAnswer::Pending`]:
/// não escolhe `allow_once` nem nenhuma outra opção.
pub fn answer(options: &[PermissionOption], choice: UserChoice) -> PermissionAnswer {
    match choice {
        UserChoice::Dismiss => PermissionAnswer::Pending,
        UserChoice::Option(index) => {
            let Some(option) = options.get(index as usize) else { return PermissionAnswer::Pending };
            selected(&option.option_id)
        }
        UserChoice::Allow => pick(options, Kind::is_allow),
        UserChoice::Reject => pick(options, Kind::is_reject),
    }
}

fn pick(options: &[PermissionOption], want: fn(Kind) -> bool) -> PermissionAnswer {
    let shown = shown_options(options);
    // Uma vez antes de sempre: é o pedido deste marco (allow_once / reject_once).
    let once = shown.iter().position(|o| matches!(o.kind, Kind::AllowOnce | Kind::RejectOnce) && want(o.kind));
    let any = shown.iter().position(|o| want(o.kind));
    let Some(index) = once.or(any) else { return PermissionAnswer::Pending };
    let Some(option) = options.get(index) else { return PermissionAnswer::Pending };
    selected(&option.option_id)
}

fn selected(id: &PermissionOptionId) -> PermissionAnswer {
    PermissionAnswer::Selected(RequestPermissionOutcome::Selected(SelectedPermissionOutcome::new(id.clone())))
}

/// `Cancel` explícito (o botão cancelar da sessão, não o Esc do diálogo).
/// Quem chama manda `session/cancel` antes de responder `cancelled`.
pub fn cancel_answer() -> PermissionAnswer {
    PermissionAnswer::Cancel
}

/// A resposta do protocolo para um [`PermissionAnswer::Cancel`].
pub fn cancelled_outcome() -> RequestPermissionOutcome {
    RequestPermissionOutcome::Cancelled
}

/// O id escolhido, se a resposta selecionou uma opção. `None` em pendente e em cancelar.
pub fn selected_id(answer: &PermissionAnswer) -> Option<String> {
    match answer {
        PermissionAnswer::Selected(RequestPermissionOutcome::Selected(s)) => Some(option_id(&s.option_id)),
        _ => None,
    }
}

/// Verdadeiro só quando a resposta é uma opção de permitir.
pub fn is_allow(options: &[PermissionOption], answer: &PermissionAnswer) -> bool {
    let Some(id) = selected_id(answer) else { return false };
    shown_options(options).into_iter().any(|o| o.id == id && o.kind.is_allow())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn opt(id: &str, name: &str, kind: PermissionOptionKind) -> PermissionOption {
        PermissionOption::new(id.to_string(), name, kind)
    }

    fn pair() -> Vec<PermissionOption> {
        vec![
            opt("allow", "Allow once", PermissionOptionKind::AllowOnce),
            opt("reject", "Reject once", PermissionOptionKind::RejectOnce),
        ]
    }

    #[test]
    fn allow_reject_and_dismiss() {
        let options = pair();
        let allow = answer(&options, UserChoice::Allow);
        assert_eq!(selected_id(&allow).as_deref(), Some("allow"));
        assert!(is_allow(&options, &allow));

        let reject = answer(&options, UserChoice::Reject);
        assert_eq!(selected_id(&reject).as_deref(), Some("reject"));
        assert!(!is_allow(&options, &reject));

        let dismiss = answer(&options, UserChoice::Dismiss);
        assert!(matches!(dismiss, PermissionAnswer::Pending));
        assert!(!is_allow(&options, &dismiss));
        assert!(selected_id(&dismiss).is_none());

        let cancel = cancel_answer();
        assert!(matches!(cancel, PermissionAnswer::Cancel));
        assert!(!is_allow(&options, &cancel));
        assert!(matches!(cancelled_outcome(), RequestPermissionOutcome::Cancelled));
    }

    #[test]
    fn once_wins_over_always_and_unknown_id_does_not_allow() {
        let options = vec![
            opt("always", "Always", PermissionOptionKind::AllowAlways),
            opt("once", "Once", PermissionOptionKind::AllowOnce),
            opt("no", "No", PermissionOptionKind::RejectAlways),
        ];
        assert_eq!(selected_id(&answer(&options, UserChoice::Allow)).as_deref(), Some("once"));
        assert_eq!(selected_id(&answer(&options, UserChoice::Reject)).as_deref(), Some("no"));
        // Índice que não existe: continua pendente, não vira a primeira opção.
        assert!(matches!(answer(&options, UserChoice::Option(9)), PermissionAnswer::Pending));
        assert_eq!(selected_id(&answer(&options, UserChoice::Option(1))).as_deref(), Some("once"));
    }

    #[test]
    fn allow_without_an_allow_option_stays_pending() {
        let options = vec![opt("no", "No", PermissionOptionKind::RejectOnce)];
        assert!(matches!(answer(&options, UserChoice::Allow), PermissionAnswer::Pending));
        assert!(!is_allow(&options, &answer(&options, UserChoice::Reject)));
    }
}
