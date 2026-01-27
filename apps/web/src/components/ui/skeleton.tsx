import { cn } from "@/lib/utils";

function Skeleton({ className, ...props }: React.ComponentProps<"div">) {
  return (
    <div
      data-slot="skeleton"
      role="status"
      aria-label="Loading content"
      aria-live="polite"
      className={cn("bg-accent animate-pulse rounded-md", className)}
      {...props}
    >
      <span className="sr-only">Loading...</span>
    </div>
  );
}

export { Skeleton };
