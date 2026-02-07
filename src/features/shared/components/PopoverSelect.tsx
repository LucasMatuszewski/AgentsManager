import { useCallback, useEffect, useRef, useState } from "react";
import type { CSSProperties } from "react";

export type PopoverSelectOption = {
  value: string;
  label: string;
};

type PopoverSelectProps = {
  options: PopoverSelectOption[];
  value: string | null;
  onChange: (value: string) => void;
  disabled?: boolean;
  ariaLabel: string;
  className?: string;
  placeholder?: string;
  width?: number;
};

export function PopoverSelect({
  options,
  value,
  onChange,
  disabled = false,
  ariaLabel,
  className = "",
  placeholder = "Select…",
  width,
}: PopoverSelectProps) {
  const [open, setOpen] = useState(false);
  const [focusIndex, setFocusIndex] = useState(-1);
  const containerRef = useRef<HTMLDivElement>(null);
  const listRef = useRef<HTMLUListElement>(null);

  const selectedOption = options.find((o) => o.value === value);
  const label = selectedOption?.label ?? placeholder;

  const close = useCallback(() => {
    setOpen(false);
    setFocusIndex(-1);
  }, []);

  // Close on outside click
  useEffect(() => {
    if (!open) return;
    const handler = (e: MouseEvent) => {
      if (
        containerRef.current &&
        !containerRef.current.contains(e.target as Node)
      ) {
        close();
      }
    };
    document.addEventListener("mousedown", handler);
    return () => document.removeEventListener("mousedown", handler);
  }, [open, close]);

  // Scroll focused item into view
  useEffect(() => {
    if (!open || focusIndex < 0 || !listRef.current) return;
    const items = listRef.current.children;
    if (items[focusIndex]) {
      (items[focusIndex] as HTMLElement).scrollIntoView({ block: "nearest" });
    }
  }, [focusIndex, open]);

  const handleKeyDown = (e: React.KeyboardEvent) => {
    if (disabled) return;

    if (!open) {
      if (e.key === "Enter" || e.key === " " || e.key === "ArrowDown") {
        e.preventDefault();
        setOpen(true);
        const idx = options.findIndex((o) => o.value === value);
        setFocusIndex(idx >= 0 ? idx : 0);
      }
      return;
    }

    switch (e.key) {
      case "Escape":
        e.preventDefault();
        close();
        break;
      case "ArrowDown":
        e.preventDefault();
        setFocusIndex((prev) =>
          prev < options.length - 1 ? prev + 1 : 0,
        );
        break;
      case "ArrowUp":
        e.preventDefault();
        setFocusIndex((prev) =>
          prev > 0 ? prev - 1 : options.length - 1,
        );
        break;
      case "Enter":
      case " ":
        e.preventDefault();
        if (focusIndex >= 0 && focusIndex < options.length) {
          onChange(options[focusIndex].value);
          close();
        }
        break;
    }
  };

  const toggle = () => {
    if (disabled) return;
    if (open) {
      close();
    } else {
      setOpen(true);
      const idx = options.findIndex((o) => o.value === value);
      setFocusIndex(idx >= 0 ? idx : 0);
    }
  };

  const style: CSSProperties = {};
  if (width) {
    style.width = width;
    style.maxWidth = width;
  }

  return (
    <div
      ref={containerRef}
      className={`popover-select ${className}`}
      style={style}
    >
      <button
        type="button"
        className="popover-select-trigger"
        aria-label={ariaLabel}
        aria-haspopup="listbox"
        aria-expanded={open}
        disabled={disabled}
        onClick={toggle}
        onKeyDown={handleKeyDown}
        style={style}
      >
        <span className="popover-select-label">{label}</span>
        <span className="popover-select-caret" aria-hidden />
      </button>
      {open && (
        <div className="popover-select-dropdown popover-surface" role="listbox">
          <ul ref={listRef} className="popover-select-list">
            {options.map((opt, i) => (
              <li
                key={opt.value}
                role="option"
                aria-selected={opt.value === value}
                className={[
                  "popover-select-item",
                  opt.value === value ? "popover-select-item--selected" : "",
                  i === focusIndex ? "popover-select-item--focused" : "",
                ]
                  .filter(Boolean)
                  .join(" ")}
                onMouseEnter={() => setFocusIndex(i)}
                onClick={() => {
                  onChange(opt.value);
                  close();
                }}
              >
                {opt.label}
              </li>
            ))}
          </ul>
        </div>
      )}
    </div>
  );
}
