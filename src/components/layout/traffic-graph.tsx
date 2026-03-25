import { forwardRef, useImperativeHandle, useRef, useState } from "react";

export interface TrafficRef {
  appendData: (data: { up: number; down: number }) => void;
  toggleStyle: () => void;
}

interface TrafficGraphProps {
  style?: React.CSSProperties;
}

const MAX_DATA_POINTS = 60;

export const TrafficGraph = forwardRef<TrafficRef, TrafficGraphProps>(
  function TrafficGraph(props, ref) {
    const [_data, _setData] = useState<{ up: number; down: number }[]>([]);
    const [_graphStyle, setGraphStyle] = useState<"area" | "line">("area");
    const canvasRef = useRef<HTMLCanvasElement>(null);

    useImperativeHandle(
      ref,
      () => ({
        appendData: (newData: { up: number; down: number }) => {
          _setData((prev) => {
            const updated = [...prev, newData];
            if (updated.length > MAX_DATA_POINTS) {
              return updated.slice(-MAX_DATA_POINTS);
            }
            return updated;
          });
        },
        toggleStyle: () => {
          setGraphStyle((prev) => (prev === "area" ? "line" : "area"));
        },
      }),
      []
    );

    return (
      <canvas
        ref={canvasRef}
        width={300}
        height={60}
        style={{
          width: "100%",
          height: "100%",
          ...props.style,
        }}
      />
    );
  }
);
