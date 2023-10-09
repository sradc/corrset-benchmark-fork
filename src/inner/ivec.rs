use super::{
  indexed::{QuestionIdx, QuestionRef, UserIdx, UserRef},
  CorrSetInner,
};
use crate::{
  utils::{self, Captures},
  Question, Row,
};
use fxhash::FxHashSet as HashSet;
use indexical::{index_vec::IndexVec, IndexedDomain};

pub struct IvecCorrSet<'a> {
  questions: IndexedDomain<QuestionRef<'a>>,
  users: IndexedDomain<UserRef<'a>>,
  q_to_score: IndexVec<QuestionIdx, IndexVec<UserIdx, Option<u32>>>,
  grand_totals: IndexVec<UserIdx, u32>,
}

impl<'a> CorrSetInner<'a> for IvecCorrSet<'a> {
  type Q = QuestionIdx;
  type Scratch = ();

  fn build(data: &'a [Row]) -> Self {
    let (users, questions): (HashSet<_>, HashSet<_>) = data
      .iter()
      .map(|row| (UserRef(&row.user), QuestionRef(&row.question)))
      .unzip();
    let users = IndexedDomain::from_iter(users);
    let questions = IndexedDomain::from_iter(questions);

    let empty_vec = IndexVec::from_iter(users.indices().map(|_| None));
    let mut q_to_score = IndexVec::from_iter(questions.indices().map(|_| empty_vec.clone()));
    for r in data {
      q_to_score[questions.index(&QuestionRef(&r.question))][users.index(&UserRef(&r.user))] =
        Some(r.score);
    }

    let grand_totals = users
      .indices()
      .map(|user| q_to_score.iter().filter_map(|v| v[user]).sum::<u32>())
      .collect::<IndexVec<_, _>>();

    IvecCorrSet {
      questions,
      users,
      q_to_score,
      grand_totals,
    }
  }

  fn iter_qs(&self) -> impl Iterator<Item = QuestionIdx> + Captures<'a> + '_ {
    self.questions.indices()
  }

  fn to_question(&self, q: Self::Q) -> &'a Question {
    self.questions.value(q).0
  }

  fn init_scratch(&self) -> Self::Scratch {}

  fn corr_set(&self, _: &mut (), qs: &[Self::Q]) -> f64 {
    let (qs_scores, grand_scores): (Vec<_>, Vec<_>) = self
      .users
      .indices()
      .filter_map(|u| {
        let total = qs
          .iter()
          .map(|q| self.q_to_score[*q][u])
          .sum::<Option<u32>>()?;
        Some((total as f64, self.grand_totals[u] as f64))
      })
      .unzip();
    utils::correlation(&qs_scores, &grand_scores)
  }
}
