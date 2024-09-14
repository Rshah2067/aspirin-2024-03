use core::num;

#[derive(PartialEq, Clone, Copy, Debug)]
enum ClassYear {
    Senior,
    Junior,
    Sophomore,
    FirstYear,
}

struct Student {
    name: &'static str,
    class_year: ClassYear,
    gpa: f32,
}

const OLIN_STUDENTS: [Student; 8] = [
    Student {
        name: "Alice",
        class_year: ClassYear::Senior,
        gpa: 3.9,
    },
    Student {
        name: "Foo",
        class_year: ClassYear::Sophomore,
        gpa: 2.3,
    },
    Student {
        name: "Bar",
        class_year: ClassYear::Junior,
        gpa: 3.9,
    },
    Student {
        name: "Ralph",
        class_year: ClassYear::Senior,
        gpa: 3.1,
    },
    Student {
        name: "Ayush",
        class_year: ClassYear::Senior,
        gpa: 0.0,
    },
    Student {
        name: "Anna",
        class_year: ClassYear::FirstYear,
        gpa: 4.0,
    },
    Student {
        name: "Hannah",
        class_year: ClassYear::FirstYear,
        gpa: 4.0,
    },
    Student{
        name: "Lorin",
        class_year: ClassYear::Junior,
        gpa: 3.6,
    }
];

fn get_average_gpa() -> f32 {
    let mut average: f32 = 0.0;
    let mut num: f32 = 0.0;
    for student in OLIN_STUDENTS {
        if student.class_year != ClassYear::FirstYear{
            average += student.gpa;
            num +=1.0;
        }
    
    }
    average/num
}

fn get_num_excel_students_for_class(class_year: ClassYear) -> u32 {
    let mut num_excel = 0;
    let average = get_average_gpa();
    for student in OLIN_STUDENTS{
        if student.class_year == class_year && student.gpa > average{
          num_excel +=1;
        }
    }
    num_excel
}

fn get_best_class() -> ClassYear {
    let average = get_average_gpa();
    let excel: [u32;3] = [get_num_excel_students_for_class(ClassYear::Sophomore),
    get_num_excel_students_for_class(ClassYear::Junior),get_num_excel_students_for_class(ClassYear::Senior)];
    let mut greatest = 0;
    for i in 1..3 {
        if excel[i] > excel[greatest]{
            greatest = i;
        }
    }
    match greatest {
        1 =>ClassYear::Sophomore,
        2 =>ClassYear::Junior,
        3 =>ClassYear::Senior,
        _ => ClassYear::Senior,
    }
}

// Do not modify below here
#[cfg(test)]
mod tests {
    use float_cmp::approx_eq;

    use crate::university::{
        get_average_gpa, get_best_class, get_num_excel_students_for_class, ClassYear,
    };

    #[test]
    fn test_get_average_gpa() {
        assert!(approx_eq!(f32, get_average_gpa(), 2.8))
    }

    #[test]
    fn test_get_num_excel_students_for_class() {
        assert_eq!(get_num_excel_students_for_class(ClassYear::Sophomore), 0);
        assert_eq!(get_num_excel_students_for_class(ClassYear::Junior), 2);
        assert_eq!(get_num_excel_students_for_class(ClassYear::Senior), 2);
    }

    #[test]
    fn test_get_best_class() {
        assert_eq!(get_best_class(), ClassYear::Senior);
    }
}
