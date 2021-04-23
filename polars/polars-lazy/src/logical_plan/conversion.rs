use crate::prelude::*;
use polars_core::prelude::*;

// converts expression to AExpr, which uses an arena (Vec) for allocation
pub(crate) fn to_aexpr(expr: Expr, arena: &mut Arena<AExpr>) -> Node {
    let v = match expr {
        Expr::IsUnique(expr) => AExpr::IsUnique(to_aexpr(*expr, arena)),
        Expr::Duplicated(expr) => AExpr::Duplicated(to_aexpr(*expr, arena)),
        Expr::Reverse(expr) => AExpr::Reverse(to_aexpr(*expr, arena)),
        Expr::Explode(expr) => AExpr::Explode(to_aexpr(*expr, arena)),
        Expr::Alias(e, name) => AExpr::Alias(to_aexpr(*e, arena), name),
        Expr::Literal(value) => AExpr::Literal(value),
        Expr::Column(s) => AExpr::Column(s),
        Expr::BinaryExpr { left, op, right } => {
            let l = to_aexpr(*left, arena);
            let r = to_aexpr(*right, arena);
            AExpr::BinaryExpr {
                left: l,
                op,
                right: r,
            }
        }
        Expr::Not(e) => AExpr::Not(to_aexpr(*e, arena)),
        Expr::IsNotNull(e) => AExpr::IsNotNull(to_aexpr(*e, arena)),
        Expr::IsNull(e) => AExpr::IsNull(to_aexpr(*e, arena)),

        Expr::Cast { expr, data_type } => AExpr::Cast {
            expr: to_aexpr(*expr, arena),
            data_type,
        },
        Expr::Sort { expr, reverse } => AExpr::Sort {
            expr: to_aexpr(*expr, arena),
            reverse,
        },
        Expr::SortBy { expr, by, reverse } => AExpr::SortBy {
            expr: to_aexpr(*expr, arena),
            by: to_aexpr(*by, arena),
            reverse,
        },
        Expr::Filter { input, by } => AExpr::Filter {
            input: to_aexpr(*input, arena),
            by: to_aexpr(*by, arena),
        },
        Expr::Agg(agg) => {
            let a_agg = match agg {
                AggExpr::Min(expr) => AAggExpr::Min(to_aexpr(*expr, arena)),
                AggExpr::Max(expr) => AAggExpr::Max(to_aexpr(*expr, arena)),
                AggExpr::Median(expr) => AAggExpr::Median(to_aexpr(*expr, arena)),
                AggExpr::NUnique(expr) => AAggExpr::NUnique(to_aexpr(*expr, arena)),
                AggExpr::First(expr) => AAggExpr::First(to_aexpr(*expr, arena)),
                AggExpr::Last(expr) => AAggExpr::Last(to_aexpr(*expr, arena)),
                AggExpr::Mean(expr) => AAggExpr::Mean(to_aexpr(*expr, arena)),
                AggExpr::List(expr) => AAggExpr::List(to_aexpr(*expr, arena)),
                AggExpr::Count(expr) => AAggExpr::Count(to_aexpr(*expr, arena)),
                AggExpr::Quantile { expr, quantile } => AAggExpr::Quantile {
                    expr: to_aexpr(*expr, arena),
                    quantile,
                },
                AggExpr::Sum(expr) => AAggExpr::Sum(to_aexpr(*expr, arena)),
                AggExpr::Std(expr) => AAggExpr::Std(to_aexpr(*expr, arena)),
                AggExpr::Var(expr) => AAggExpr::Var(to_aexpr(*expr, arena)),
                AggExpr::AggGroups(expr) => AAggExpr::AggGroups(to_aexpr(*expr, arena)),
            };
            AExpr::Agg(a_agg)
        }
        Expr::Ternary {
            predicate,
            truthy,
            falsy,
        } => {
            let p = to_aexpr(*predicate, arena);
            let t = to_aexpr(*truthy, arena);
            let f = to_aexpr(*falsy, arena);
            AExpr::Ternary {
                predicate: p,
                truthy: t,
                falsy: f,
            }
        }
        Expr::Udf {
            input,
            function,
            output_type,
        } => AExpr::Udf {
            input: to_aexpr(*input, arena),
            function,
            output_type,
        },
        Expr::BinaryFunction {
            input_a,
            input_b,
            function,
            output_field,
        } => AExpr::BinaryFunction {
            input_a: to_aexpr(*input_a, arena),
            input_b: to_aexpr(*input_b, arena),
            function,
            output_field,
        },
        Expr::Shift { input, periods } => AExpr::Shift {
            input: to_aexpr(*input, arena),
            periods,
        },
        Expr::Window {
            function,
            partition_by,
            order_by,
        } => AExpr::Window {
            function: to_aexpr(*function, arena),
            partition_by: to_aexpr(*partition_by, arena),
            order_by: order_by.map(|ob| to_aexpr(*ob, arena)),
        },
        Expr::Slice {
            input,
            offset,
            length,
        } => AExpr::Slice {
            input: to_aexpr(*input, arena),
            offset,
            length,
        },
        Expr::Wildcard => AExpr::Wildcard,
        Expr::Except(input) => AExpr::Except(to_aexpr(*input, arena)),
    };
    arena.add(v)
}

pub(crate) fn to_alp(
    lp: LogicalPlan,
    expr_arena: &mut Arena<AExpr>,
    lp_arena: &mut Arena<ALogicalPlan>,
) -> Node {
    let v = match lp {
        LogicalPlan::Selection { input, predicate } => {
            let i = to_alp(*input, expr_arena, lp_arena);
            let p = to_aexpr(predicate, expr_arena);
            ALogicalPlan::Selection {
                input: i,
                predicate: p,
            }
        }
        LogicalPlan::Slice { input, offset, len } => {
            let input = to_alp(*input, expr_arena, lp_arena);
            ALogicalPlan::Slice { input, offset, len }
        }
        LogicalPlan::Melt {
            input,
            id_vars,
            value_vars,
            schema,
        } => {
            let input = to_alp(*input, expr_arena, lp_arena);
            ALogicalPlan::Melt {
                input,
                id_vars,
                value_vars,
                schema,
            }
        }
        LogicalPlan::CsvScan {
            path,
            schema,
            has_header,
            delimiter,
            ignore_errors,
            skip_rows,
            stop_after_n_rows,
            with_columns,
            predicate,
            aggregate,
            cache,
        } => ALogicalPlan::CsvScan {
            path,
            schema,
            has_header,
            delimiter,
            ignore_errors,
            skip_rows,
            stop_after_n_rows,
            with_columns,
            predicate: predicate.map(|expr| to_aexpr(expr, expr_arena)),
            aggregate: aggregate
                .into_iter()
                .map(|expr| to_aexpr(expr, expr_arena))
                .collect(),
            cache,
        },
        #[cfg(feature = "parquet")]
        LogicalPlan::ParquetScan {
            path,
            schema,
            with_columns,
            predicate,
            aggregate,
            stop_after_n_rows,
            cache,
        } => ALogicalPlan::ParquetScan {
            path,
            schema,
            with_columns,
            predicate: predicate.map(|expr| to_aexpr(expr, expr_arena)),
            aggregate: aggregate
                .into_iter()
                .map(|expr| to_aexpr(expr, expr_arena))
                .collect(),
            stop_after_n_rows,
            cache,
        },
        LogicalPlan::DataFrameScan {
            df,
            schema,
            projection,
            selection,
        } => ALogicalPlan::DataFrameScan {
            df,
            schema,
            projection: projection
                .map(|exprs| exprs.into_iter().map(|x| to_aexpr(x, expr_arena)).collect()),
            selection: selection.map(|expr| to_aexpr(expr, expr_arena)),
        },
        LogicalPlan::Projection {
            expr,
            input,
            schema,
        } => {
            let exp = expr.into_iter().map(|x| to_aexpr(x, expr_arena)).collect();
            let i = to_alp(*input, expr_arena, lp_arena);
            ALogicalPlan::Projection {
                expr: exp,
                input: i,
                schema,
            }
        }
        LogicalPlan::LocalProjection {
            expr,
            input,
            schema,
        } => {
            let exp = expr.into_iter().map(|x| to_aexpr(x, expr_arena)).collect();
            let i = to_alp(*input, expr_arena, lp_arena);
            ALogicalPlan::LocalProjection {
                expr: exp,
                input: i,
                schema,
            }
        }
        LogicalPlan::Sort {
            input,
            by_column,
            reverse,
        } => {
            let input = to_alp(*input, expr_arena, lp_arena);
            ALogicalPlan::Sort {
                input,
                by_column,
                reverse,
            }
        }
        LogicalPlan::Explode { input, columns } => {
            let input = to_alp(*input, expr_arena, lp_arena);
            ALogicalPlan::Explode { input, columns }
        }
        LogicalPlan::Cache { input } => {
            let input = to_alp(*input, expr_arena, lp_arena);
            ALogicalPlan::Cache { input }
        }
        LogicalPlan::Aggregate {
            input,
            keys,
            aggs,
            schema,
            apply,
        } => {
            let i = to_alp(*input, expr_arena, lp_arena);
            let aggs_new = aggs.into_iter().map(|x| to_aexpr(x, expr_arena)).collect();
            let keys_new = keys
                .iter()
                .map(|x| to_aexpr(x.clone(), expr_arena))
                .collect();

            ALogicalPlan::Aggregate {
                input: i,
                keys: keys_new,
                aggs: aggs_new,
                schema,
                apply,
            }
        }
        LogicalPlan::Join {
            input_left,
            input_right,
            schema,
            how,
            left_on,
            right_on,
            allow_par,
            force_par,
        } => {
            let i_l = to_alp(*input_left, expr_arena, lp_arena);
            let i_r = to_alp(*input_right, expr_arena, lp_arena);

            let l_on = left_on
                .into_iter()
                .map(|x| to_aexpr(x, expr_arena))
                .collect();
            let r_on = right_on
                .into_iter()
                .map(|x| to_aexpr(x, expr_arena))
                .collect();

            ALogicalPlan::Join {
                input_left: i_l,
                input_right: i_r,
                schema,
                left_on: l_on,
                how,
                right_on: r_on,
                allow_par,
                force_par,
            }
        }
        LogicalPlan::HStack {
            input,
            exprs,
            schema,
        } => {
            let exp = exprs.into_iter().map(|x| to_aexpr(x, expr_arena)).collect();
            let i = to_alp(*input, expr_arena, lp_arena);
            ALogicalPlan::HStack {
                input: i,
                exprs: exp,
                schema,
            }
        }
        LogicalPlan::Distinct {
            input,
            maintain_order,
            subset,
        } => {
            let i = to_alp(*input, expr_arena, lp_arena);
            ALogicalPlan::Distinct {
                input: i,
                maintain_order,
                subset,
            }
        }
        LogicalPlan::Udf {
            input,
            function,
            projection_pd,
            predicate_pd,
            schema,
        } => {
            let input = to_alp(*input, expr_arena, lp_arena);
            ALogicalPlan::Udf {
                input,
                function,
                projection_pd,
                predicate_pd,
                schema,
            }
        }
    };
    lp_arena.add(v)
}

pub(crate) fn node_to_exp(node: Node, expr_arena: &Arena<AExpr>) -> Expr {
    let expr = expr_arena.get(node).clone();

    match expr {
        AExpr::Duplicated(node) => Expr::Duplicated(Box::new(node_to_exp(node, expr_arena))),
        AExpr::IsUnique(node) => Expr::IsUnique(Box::new(node_to_exp(node, expr_arena))),
        AExpr::Reverse(node) => Expr::Reverse(Box::new(node_to_exp(node, expr_arena))),
        AExpr::Explode(node) => Expr::Explode(Box::new(node_to_exp(node, expr_arena))),
        AExpr::Alias(expr, name) => {
            let exp = node_to_exp(expr, expr_arena);
            Expr::Alias(Box::new(exp), name)
        }
        AExpr::Column(a) => Expr::Column(a),
        AExpr::Literal(s) => Expr::Literal(s),
        AExpr::BinaryExpr { left, op, right } => {
            let l = node_to_exp(left, expr_arena);
            let r = node_to_exp(right, expr_arena);
            Expr::BinaryExpr {
                left: Box::new(l),
                op,
                right: Box::new(r),
            }
        }
        AExpr::Not(expr) => {
            let exp = node_to_exp(expr, expr_arena);
            Expr::Not(Box::new(exp))
        }
        AExpr::IsNotNull(expr) => {
            let exp = node_to_exp(expr, expr_arena);
            Expr::IsNotNull(Box::new(exp))
        }
        AExpr::IsNull(expr) => {
            let exp = node_to_exp(expr, expr_arena);
            Expr::IsNull(Box::new(exp))
        }
        AExpr::Cast { expr, data_type } => {
            let exp = node_to_exp(expr, expr_arena);
            Expr::Cast {
                expr: Box::new(exp),
                data_type,
            }
        }
        AExpr::Sort { expr, reverse } => {
            let exp = node_to_exp(expr, expr_arena);
            Expr::Sort {
                expr: Box::new(exp),
                reverse,
            }
        }
        AExpr::SortBy { expr, by, reverse } => {
            let expr = node_to_exp(expr, expr_arena);
            let by = node_to_exp(by, expr_arena);
            Expr::SortBy {
                expr: Box::new(expr),
                by: Box::new(by),
                reverse,
            }
        }
        AExpr::Filter { input, by } => {
            let input = node_to_exp(input, expr_arena);
            let by = node_to_exp(by, expr_arena);
            Expr::Filter {
                input: Box::new(input),
                by: Box::new(by),
            }
        }
        AExpr::Agg(agg) => match agg {
            AAggExpr::Min(expr) => {
                let exp = node_to_exp(expr, expr_arena);
                AggExpr::Min(Box::new(exp)).into()
            }
            AAggExpr::Max(expr) => {
                let exp = node_to_exp(expr, expr_arena);
                AggExpr::Max(Box::new(exp)).into()
            }

            AAggExpr::Median(expr) => {
                let exp = node_to_exp(expr, expr_arena);
                AggExpr::Median(Box::new(exp)).into()
            }
            AAggExpr::NUnique(expr) => {
                let exp = node_to_exp(expr, expr_arena);
                AggExpr::NUnique(Box::new(exp)).into()
            }
            AAggExpr::First(expr) => {
                let exp = node_to_exp(expr, expr_arena);
                AggExpr::First(Box::new(exp)).into()
            }
            AAggExpr::Last(expr) => {
                let exp = node_to_exp(expr, expr_arena);
                AggExpr::Last(Box::new(exp)).into()
            }
            AAggExpr::Mean(expr) => {
                let exp = node_to_exp(expr, expr_arena);
                AggExpr::Mean(Box::new(exp)).into()
            }
            AAggExpr::List(expr) => {
                let exp = node_to_exp(expr, expr_arena);
                AggExpr::List(Box::new(exp)).into()
            }
            AAggExpr::Quantile { expr, quantile } => {
                let exp = node_to_exp(expr, expr_arena);
                AggExpr::Quantile {
                    expr: Box::new(exp),
                    quantile,
                }
                .into()
            }
            AAggExpr::Sum(expr) => {
                let exp = node_to_exp(expr, expr_arena);
                AggExpr::Sum(Box::new(exp)).into()
            }
            AAggExpr::Std(expr) => {
                let exp = node_to_exp(expr, expr_arena);
                AggExpr::Std(Box::new(exp)).into()
            }
            AAggExpr::Var(expr) => {
                let exp = node_to_exp(expr, expr_arena);
                AggExpr::Var(Box::new(exp)).into()
            }
            AAggExpr::AggGroups(expr) => {
                let exp = node_to_exp(expr, expr_arena);
                AggExpr::AggGroups(Box::new(exp)).into()
            }
            AAggExpr::Count(expr) => {
                let exp = node_to_exp(expr, expr_arena);
                AggExpr::Count(Box::new(exp)).into()
            }
        },
        AExpr::Shift { input, periods } => {
            let e = node_to_exp(input, expr_arena);
            Expr::Shift {
                input: Box::new(e),
                periods,
            }
        }
        AExpr::Ternary {
            predicate,
            truthy,
            falsy,
        } => {
            let p = node_to_exp(predicate, expr_arena);
            let t = node_to_exp(truthy, expr_arena);
            let f = node_to_exp(falsy, expr_arena);

            Expr::Ternary {
                predicate: Box::new(p),
                truthy: Box::new(t),
                falsy: Box::new(f),
            }
        }
        AExpr::Udf {
            input,
            function,
            output_type,
        } => {
            let i = node_to_exp(input, expr_arena);
            Expr::Udf {
                input: Box::new(i),
                function,
                output_type,
            }
        }
        AExpr::BinaryFunction {
            input_a,
            input_b,
            function,
            output_field,
        } => Expr::BinaryFunction {
            input_a: Box::new(node_to_exp(input_a, expr_arena)),
            input_b: Box::new(node_to_exp(input_b, expr_arena)),
            function,
            output_field,
        },
        AExpr::Window {
            function,
            partition_by,
            order_by,
        } => {
            let function = Box::new(node_to_exp(function, expr_arena));
            let partition_by = Box::new(node_to_exp(partition_by, expr_arena));
            let order_by = order_by.map(|ob| Box::new(node_to_exp(ob, expr_arena)));
            Expr::Window {
                function,
                partition_by,
                order_by,
            }
        }
        AExpr::Slice {
            input,
            offset,
            length,
        } => Expr::Slice {
            input: Box::new(node_to_exp(input, expr_arena)),
            offset,
            length,
        },
        AExpr::Wildcard => Expr::Wildcard,
        AExpr::Except(node) => Expr::Except(Box::new(node_to_exp(node, expr_arena))),
    }
}

pub(crate) fn node_to_lp(
    node: Node,
    expr_arena: &mut Arena<AExpr>,
    lp_arena: &mut Arena<ALogicalPlan>,
) -> LogicalPlan {
    let lp = lp_arena.get_mut(node);
    let lp = std::mem::take(lp);

    match lp {
        ALogicalPlan::Slice { input, offset, len } => {
            let lp = node_to_lp(input, expr_arena, lp_arena);
            LogicalPlan::Slice {
                input: Box::new(lp),
                offset,
                len,
            }
        }
        ALogicalPlan::Selection { input, predicate } => {
            let lp = node_to_lp(input, expr_arena, lp_arena);
            let p = node_to_exp(predicate, expr_arena);
            LogicalPlan::Selection {
                input: Box::new(lp),
                predicate: p,
            }
        }
        ALogicalPlan::CsvScan {
            path,
            schema,
            has_header,
            delimiter,
            ignore_errors,
            skip_rows,
            stop_after_n_rows,
            with_columns,
            predicate,
            aggregate,
            cache,
        } => LogicalPlan::CsvScan {
            path,
            schema,
            has_header,
            delimiter,
            ignore_errors,
            skip_rows,
            stop_after_n_rows,
            with_columns,
            predicate: predicate.map(|n| node_to_exp(n, expr_arena)),
            aggregate: aggregate
                .into_iter()
                .map(|n| node_to_exp(n, expr_arena))
                .collect(),
            cache,
        },
        #[cfg(feature = "parquet")]
        ALogicalPlan::ParquetScan {
            path,
            schema,
            with_columns,
            predicate,
            aggregate,
            stop_after_n_rows,
            cache,
        } => LogicalPlan::ParquetScan {
            path,
            schema,
            with_columns,
            predicate: predicate.map(|n| node_to_exp(n, expr_arena)),
            aggregate: aggregate
                .into_iter()
                .map(|n| node_to_exp(n, expr_arena))
                .collect(),
            stop_after_n_rows,
            cache,
        },
        ALogicalPlan::DataFrameScan {
            df,
            schema,
            projection,
            selection,
        } => LogicalPlan::DataFrameScan {
            df,
            schema,
            projection: projection
                .as_ref()
                .map(|nodes| nodes.iter().map(|n| node_to_exp(*n, expr_arena)).collect()),
            selection: selection.map(|n| node_to_exp(n, expr_arena)),
        },
        ALogicalPlan::Projection {
            expr,
            input,
            schema,
        } => {
            let exprs = expr.iter().map(|x| node_to_exp(*x, expr_arena)).collect();
            let i = node_to_lp(input, expr_arena, lp_arena);

            LogicalPlan::Projection {
                expr: exprs,
                input: Box::new(i),
                schema,
            }
        }
        ALogicalPlan::LocalProjection {
            expr,
            input,
            schema,
        } => {
            let exprs = expr.iter().map(|x| node_to_exp(*x, expr_arena)).collect();
            let i = node_to_lp(input, expr_arena, lp_arena);

            LogicalPlan::LocalProjection {
                expr: exprs,
                input: Box::new(i),
                schema,
            }
        }
        ALogicalPlan::Sort {
            input,
            by_column,
            reverse,
        } => {
            let input = Box::new(node_to_lp(input, expr_arena, lp_arena));
            LogicalPlan::Sort {
                input,
                by_column,
                reverse,
            }
        }
        ALogicalPlan::Explode { input, columns } => {
            let input = Box::new(node_to_lp(input, expr_arena, lp_arena));
            LogicalPlan::Explode { input, columns }
        }
        ALogicalPlan::Cache { input } => {
            let input = Box::new(node_to_lp(input, expr_arena, lp_arena));
            LogicalPlan::Cache { input }
        }
        ALogicalPlan::Aggregate {
            input,
            keys,
            aggs,
            schema,
            apply,
        } => {
            let i = node_to_lp(input, expr_arena, lp_arena);
            let a = aggs.iter().map(|x| node_to_exp(*x, expr_arena)).collect();
            let keys = Arc::new(keys.iter().map(|x| node_to_exp(*x, expr_arena)).collect());

            LogicalPlan::Aggregate {
                input: Box::new(i),
                keys,
                aggs: a,
                schema,
                apply,
            }
        }
        ALogicalPlan::Join {
            input_left,
            input_right,
            schema,
            how,
            left_on,
            right_on,
            allow_par,
            force_par,
        } => {
            let i_l = node_to_lp(input_left, expr_arena, lp_arena);
            let i_r = node_to_lp(input_right, expr_arena, lp_arena);

            let l_on = left_on
                .into_iter()
                .map(|n| node_to_exp(n, expr_arena))
                .collect();
            let r_on = right_on
                .into_iter()
                .map(|n| node_to_exp(n, expr_arena))
                .collect();

            LogicalPlan::Join {
                input_left: Box::new(i_l),
                input_right: Box::new(i_r),
                schema,
                how,
                left_on: l_on,
                right_on: r_on,
                allow_par,
                force_par,
            }
        }
        ALogicalPlan::HStack {
            input,
            exprs,
            schema,
        } => {
            let i = node_to_lp(input, expr_arena, lp_arena);
            let e = exprs.iter().map(|x| node_to_exp(*x, expr_arena)).collect();

            LogicalPlan::HStack {
                input: Box::new(i),
                exprs: e,
                schema,
            }
        }
        ALogicalPlan::Distinct {
            input,
            maintain_order,
            subset,
        } => {
            let i = node_to_lp(input, expr_arena, lp_arena);
            LogicalPlan::Distinct {
                input: Box::new(i),
                maintain_order,
                subset,
            }
        }
        ALogicalPlan::Melt {
            input,
            id_vars,
            value_vars,
            schema,
        } => {
            let input = node_to_lp(input, expr_arena, lp_arena);
            LogicalPlan::Melt {
                input: Box::new(input),
                id_vars,
                value_vars,
                schema,
            }
        }
        ALogicalPlan::Udf {
            input,
            function,
            predicate_pd,
            projection_pd,
            schema,
        } => {
            let input = Box::new(node_to_lp(input, expr_arena, lp_arena));
            LogicalPlan::Udf {
                input,
                function,
                predicate_pd,
                projection_pd,
                schema,
            }
        }
    }
}