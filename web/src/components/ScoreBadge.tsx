import { TONE_LABEL, scoreTone } from "../format";

export function ScoreBadge({ score }: { score: number }) {
  const tone = scoreTone(score);
  return (
    <span className={`score score-${tone}`}>
      <span className="score-value">{score}</span>
      <span className="score-label">{TONE_LABEL[tone]}</span>
    </span>
  );
}
