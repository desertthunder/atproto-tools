import { Position, type InternalNode, type Node, type XYPosition } from '@xyflow/svelte';

import { SOCIAL_NODE_HEIGHT, SOCIAL_NODE_WIDTH } from '$lib/graph/layout';

type NodeBox = XYPosition & { height: number; width: number };

export type FloatingEdgeParams = {
  sourcePosition: Position;
  sourceX: number;
  sourceY: number;
  targetPosition: Position;
  targetX: number;
  targetY: number;
};

export const getFloatingEdgeParams = <NodeType extends Node>(
  sourceNode: InternalNode<NodeType>,
  targetNode: InternalNode<NodeType>
): FloatingEdgeParams => {
  const sourceBox = getNodeBox(sourceNode);
  const targetBox = getNodeBox(targetNode);
  const sourcePoint = getNodeIntersection(sourceBox, targetBox);
  const targetPoint = getNodeIntersection(targetBox, sourceBox);

  return {
    sourceX: sourcePoint.x,
    sourceY: sourcePoint.y,
    targetX: targetPoint.x,
    targetY: targetPoint.y,
    sourcePosition: getEdgePosition(sourceBox, sourcePoint),
    targetPosition: getEdgePosition(targetBox, targetPoint)
  };
};

const getNodeBox = <NodeType extends Node>(node: InternalNode<NodeType>): NodeBox => {
  const width = node.measured.width ?? node.width ?? node.initialWidth ?? SOCIAL_NODE_WIDTH;
  const height = node.measured.height ?? node.height ?? node.initialHeight ?? SOCIAL_NODE_HEIGHT;
  const position = node.internals.positionAbsolute ?? {
    x: node.position.x - width / 2,
    y: node.position.y - height / 2
  };

  return { ...position, width, height };
};

const getNodeCenter = (box: NodeBox): XYPosition => ({ x: box.x + box.width / 2, y: box.y + box.height / 2 });

const getNodeIntersection = (source: NodeBox, target: NodeBox): XYPosition => {
  const sourceCenter = getNodeCenter(source);
  const targetCenter = getNodeCenter(target);
  const deltaX = targetCenter.x - sourceCenter.x;
  const deltaY = targetCenter.y - sourceCenter.y;

  if (deltaX === 0 && deltaY === 0) return sourceCenter;

  const scaleX = deltaX === 0 ? Number.POSITIVE_INFINITY : source.width / 2 / Math.abs(deltaX);
  const scaleY = deltaY === 0 ? Number.POSITIVE_INFINITY : source.height / 2 / Math.abs(deltaY);
  const scale = Math.min(scaleX, scaleY);

  return { x: sourceCenter.x + deltaX * scale, y: sourceCenter.y + deltaY * scale };
};

const getEdgePosition = (box: NodeBox, point: XYPosition) => {
  const roundedX = Math.round(point.x);
  const roundedY = Math.round(point.y);

  if (roundedX <= Math.round(box.x) + 1) return Position.Left;
  if (roundedX >= Math.round(box.x + box.width) - 1) return Position.Right;
  if (roundedY <= Math.round(box.y) + 1) return Position.Top;
  if (roundedY >= Math.round(box.y + box.height) - 1) return Position.Bottom;

  return Position.Top;
};
