import type { SidebarNodeSummary } from '@mediaplayernext/contracts'

export type SidebarLabelDisplayMode = 'full' | 'leaf'

interface SidebarTreeNode {
  node: SidebarNodeSummary
  children: SidebarTreeNode[]
}

function isFolderNode(node: SidebarNodeSummary): boolean {
  return node.nodeType === 'folder'
}

function isMediaNode(node: SidebarNodeSummary): boolean {
  return node.nodeType === 'media_source'
}

function buildSidebarTree(nodes: SidebarNodeSummary[]): SidebarTreeNode[] {
  const treeNodeById = new Map<string, SidebarTreeNode>()

  for (const node of nodes) {
    treeNodeById.set(node.nodeId, {
      node,
      children: [],
    })
  }

  const roots: SidebarTreeNode[] = []
  for (const treeNode of treeNodeById.values()) {
    const parentNodeId = treeNode.node.parentNodeId
    if (typeof parentNodeId === 'string') {
      const parent = treeNodeById.get(parentNodeId)
      if (parent) {
        parent.children.push(treeNode)
        continue
      }
    }

    roots.push(treeNode)
  }

  return roots
}

function sortChildrenMediaFirst(nodes: SidebarTreeNode[]): SidebarTreeNode[] {
  return nodes
    .map((treeNode) => ({
      ...treeNode,
      children: sortChildrenMediaFirst(treeNode.children),
    }))
    .sort((left, right) => {
      const leftIsMedia = isMediaNode(left.node)
      const rightIsMedia = isMediaNode(right.node)
      if (leftIsMedia !== rightIsMedia) {
        return leftIsMedia ? -1 : 1
      }

      return left.node.label.localeCompare(right.node.label, 'zh-CN', {
        numeric: true,
        sensitivity: 'base',
      })
    })
}

function compactSingleFolderChain(nodes: SidebarTreeNode[]): SidebarTreeNode[] {
  const compactNode = (node: SidebarTreeNode): SidebarTreeNode => {
    let cursor = node
    const mergedLabels = [node.node.label]

    while (isFolderNode(cursor.node) && cursor.children.length === 1) {
      const onlyChild = cursor.children[0]
      if (isMediaNode(onlyChild.node)) {
        break
      }
      mergedLabels.push(onlyChild.node.label)
      cursor = onlyChild
      if (!isFolderNode(cursor.node)) {
        break
      }
    }

    return {
      ...cursor,
      node: {
        ...cursor.node,
        label: mergedLabels.length > 1 ? mergedLabels.join('/') : cursor.node.label,
      },
      children: compactSingleFolderChain(cursor.children),
    }
  }

  return nodes.map((node) => compactNode(node))
}

function pruneProceduralFolderNodes(nodes: SidebarTreeNode[]): SidebarTreeNode[] {
  const next: SidebarTreeNode[] = []

  for (const treeNode of nodes) {
    const normalizedChildren = pruneProceduralFolderNodes(treeNode.children)
    const normalizedNode: SidebarTreeNode = {
      ...treeNode,
      children: normalizedChildren,
    }

    const hasDirectMediaChild = normalizedChildren.some((child) => isMediaNode(child.node))
    if (isFolderNode(normalizedNode.node) && !hasDirectMediaChild) {
      next.push(...normalizedChildren)
      continue
    }

    next.push(normalizedNode)
  }

  return next
}

function normalizePointerFolderLabel(node: SidebarNodeSummary): SidebarNodeSummary {
  if (!isFolderNode(node)) {
    return node
  }

  return {
    ...node,
    label: node.treePath.join('/'),
  }
}

function normalizePointerFolderLabels(nodes: SidebarTreeNode[]): SidebarTreeNode[] {
  return nodes.map((treeNode) => ({
    ...treeNode,
    node: normalizePointerFolderLabel(treeNode.node),
    children: normalizePointerFolderLabels(treeNode.children),
  }))
}

function flattenTree(nodes: SidebarTreeNode[]): SidebarNodeSummary[] {
  const flattened: SidebarNodeSummary[] = []

  const walk = (treeNode: SidebarTreeNode, depth: number, parentNodeId?: string) => {
    const hasDirectMediaChild = treeNode.children.some((child) => isMediaNode(child.node))
    flattened.push({
      ...treeNode.node,
      depth,
      parentNodeId,
      hasDirectMediaChild,
    })

    for (const child of treeNode.children) {
      walk(child, depth + 1, treeNode.node.nodeId)
    }
  }

  for (const node of nodes) {
    walk(node, 0)
  }

  return flattened
}

export function normalizeSidebarImageNodes(rawNodes: SidebarNodeSummary[]): SidebarNodeSummary[] {
  const roots = buildSidebarTree(rawNodes)
  const orderedTree = sortChildrenMediaFirst(roots)
  const compactedTree = compactSingleFolderChain(orderedTree)
  const prunedTree = pruneProceduralFolderNodes(compactedTree)
  const normalizedTree = sortChildrenMediaFirst(normalizePointerFolderLabels(prunedTree))
  return flattenTree(normalizedTree)
}
