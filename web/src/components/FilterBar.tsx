import { NO_FILTER, isFiltered, parseCategory, parseSeverity, type FindingFilter } from "../filters";
import { capitalize } from "../format";
import { CATEGORIES, SEVERITIES } from "../types";

interface FilterBarProps {
  filter: FindingFilter;
  onChange: (next: FindingFilter) => void;
  total: number;
  shown: number;
}

export function FilterBar({ filter, onChange, total, shown }: FilterBarProps) {
  return (
    <div className="filters" role="search" aria-label="Filter findings">
      <label className="field">
        <span>Search</span>
        <input
          type="search"
          value={filter.query}
          placeholder="Rule, file or text"
          onChange={(event) => onChange({ ...filter, query: event.target.value })}
        />
      </label>
      <label className="field">
        <span>Category</span>
        <select
          value={filter.category}
          onChange={(event) => onChange({ ...filter, category: parseCategory(event.target.value) })}
        >
          <option value="all">All categories</option>
          {CATEGORIES.map((category) => (
            <option key={category} value={category}>
              {capitalize(category)}
            </option>
          ))}
        </select>
      </label>
      <label className="field">
        <span>Severity</span>
        <select
          value={filter.severity}
          onChange={(event) => onChange({ ...filter, severity: parseSeverity(event.target.value) })}
        >
          <option value="all">All severities</option>
          {[...SEVERITIES].reverse().map((severity) => (
            <option key={severity} value={severity}>
              {capitalize(severity)}
            </option>
          ))}
        </select>
      </label>
      <p className="filter-count" aria-live="polite">
        {`Showing ${shown} of ${total}`}
      </p>
      {isFiltered(filter) ? (
        <button type="button" className="button button-quiet" onClick={() => onChange(NO_FILTER)}>
          Clear filters
        </button>
      ) : null}
    </div>
  );
}
