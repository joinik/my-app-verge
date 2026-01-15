import { DraggableAttributes, DraggableSyntheticListeners } from "@dnd-kit/core";
import type { CSSProperties, ReactNode } from "react"


interface SortableProps {
  setNodeRef?: (element: HTMLElement | null) => void;
  attributes?: DraggableAttributes;
  listeners?: DraggableSyntheticListeners;
  style?: CSSProperties;
  isDragging?: boolean;
  disabled?: boolean;
}

interface Props {
  to: string
  children: string
  icon: ReactNode[]
  sortable?: SortableProps
}

export const LayoutItem = (props: Props) => {
  const { to, children, icon, sortable } = props;
  const { verge } = useVerge();
  const { menu_icon } = verge ?? {};
  const navCollapsed = verge?.collapse_navbar ?? false;
  const resolved = useResolvedPath(to);
  const match = useMatch({ path: resolved.pathname, end: true });
  const navigate = useNavigate();

  const effectiveMenuIcon =
    navCollapsed && menu_icon === "disable" ? "monochrome" : menu_icon;

  const { setNodeRef, attributes, listeners, style, isDragging, disabled } =
    sortable ?? {};

  const draggable = Boolean(sortable) && !disabled;
  const dragHandleProps = draggable
    ? { ...(attributes ?? {}), ...(listeners ?? {}) }
    : undefined;

  return (
    <ListItem
      ref={setNodeRef}
      style={style}
      sx={[
export const Lay